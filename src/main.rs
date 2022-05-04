mod torrent_list_model;
mod transmission_client;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Application};
use glib::clone;
use gtk::glib;
use gtk::gio;
use torrent_list_model::TorrentInfo;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let app = gtk::Application::new(
        Some("org.example.HelloWorld"),
        Default::default());
    
    //let r =&model;
 // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}    

fn build_ui(app: &Application) {

    let  model = gio::ListStore::new(TorrentInfo::static_type());
    let window = gtk::ApplicationWindow::new(app);
    window.set_default_size(400, 400);
    window.set_title(Some("Search Bar"));

    let header_bar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&header_bar));

    let search_button = gtk::ToggleButton::new();
    search_button.set_icon_name("system-search-symbolic");
    header_bar.pack_end(&search_button);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.set_child(Some(&container));

    let search_bar = gtk::SearchBar::builder()
        .valign(gtk::Align::Start)
        .key_capture_widget(&window)
        .build();

    container.append(&search_bar);

    search_button
        .bind_property("active", &search_bar, "search-mode-enabled")
        .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
        .build();

    let entry = gtk::SearchEntry::new();
    entry.set_hexpand(true);
    search_bar.set_child(Some(&entry));

    // TODO: move to a separate file
    // TODO: add icons!
    let details_notebook = gtk::Notebook::builder()
        .build();

    details_notebook.append_page(&gtk::Box::new(gtk::Orientation::Horizontal, 6), Some(&gtk::Label::new(Some("General"))));
    details_notebook.append_page(&gtk::Box::new(gtk::Orientation::Horizontal, 6), Some(&gtk::Label::new(Some("Trackers"))));
    details_notebook.append_page(&gtk::Box::new(gtk::Orientation::Horizontal, 6), Some(&gtk::Label::new(Some("Peers"))));
    details_notebook.append_page(&gtk::Box::new(gtk::Orientation::Horizontal, 6), Some(&gtk::Label::new(Some("Files"))));
    details_notebook.append_page(&gtk::Box::new(gtk::Orientation::Horizontal, 6), Some(&gtk::Label::new(Some("Statistics"))));

    let label1 = gtk::Label::builder()
        .label("Left pane yeah")
        .vexpand(true)
        .hexpand(false)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(vec!["large-title".to_string()])
        .build();
    
   
    let visible_torrents_mutex : Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
    let visible_torrents_mutex_rm = visible_torrents_mutex.clone();
    let visible_torrents_mutex_read = visible_torrents_mutex.clone();
    //        fucking transmission get torrents 
    {
        use transmission_rpc::types::{BasicAuth, RpcResponse};
        use transmission_rpc::types::{Torrent, TorrentGetField, Torrents, Id, IdOrRecent};
        use transmission_rpc::TransClient;
        use std::thread;

        
        enum TorrentUpdate {
          Full(Vec<Torrent>),
          Partial(Vec<Torrent>)
        }

        

        use glib::{MainContext, PRIORITY_DEFAULT};

        let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT); 
        //let main_context = MainContext::default();

        rx.attach(None, clone!(@weak model => @default-return Continue(false), move |update:TorrentUpdate|{
          match update {
            TorrentUpdate::Full(xs) => {
              let torrents: Vec<TorrentInfo> = xs
              .iter()
              .map(|it| TorrentInfo::new(
                  it.id.unwrap_or_default(),
                  it.name.as_ref().unwrap().to_string(), 
                  it.status.unwrap_or_default(),
                  it.percent_done.unwrap_or_default(),
                  it.rate_upload.unwrap_or_default(),
                  it.total_size.unwrap_or_default(),
                  it.download_dir.as_ref().unwrap().to_string(),
                  it.added_date.unwrap_or_default())
                ).collect();
              model.splice(0, 0, &torrents);
              },
           TorrentUpdate::Partial(xs) => {
                  println!(">>update received "); // FIXME: somehow merge this two fucking lists..
                  for x in xs {
                      let mut i = 0;
                      while let Some(y) = model.item(i) {
                        if x.id.unwrap_or(-1) == y.property_value("id").get::<i64>().expect("skdfj") {
                     //     println!("updating id: {}", x.id.unwrap_or(-1));
                          y.set_property("rate-upload", x.rate_upload.unwrap_or_default().to_value());
                          break;
                        }
                        i+=1;
                      }
                  } 

                  // how to update those bloody items?
                  }
              }
          }

           Continue(true)
        ));

        let client = TransClient::with_auth(&"http://192.168.1.217:9091/transmission/rpc".to_string(), 
                                            BasicAuth {user: "transmission".to_string(), password: "transmission".to_string()});
        thread::spawn(move || {
          use tokio::runtime::Runtime;

          let rt = Runtime::new().expect("create tokio runtime");
          rt.block_on(async {

                let fields = Some(vec![TorrentGetField::Id, TorrentGetField::Name, TorrentGetField::Status,
                            TorrentGetField::Percentdone, TorrentGetField::Rateupload, 
                            TorrentGetField::Totalsize, TorrentGetField::Downloaddir,
                            TorrentGetField::Addeddate]);
                let foo = client.torrent_get(fields, None).await.expect("Call Failed!").arguments.torrents;
                tx.send(TorrentUpdate::Full(foo)).expect("blah");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

              loop {
                let xs = visible_torrents_mutex_read.lock().unwrap();//.push(id);
                let xss: Vec<Id> = xs.iter().take(10).map(|x| Id::Id(*x)).collect();                                                                     
                drop(xs);
                let fields = Some(vec![TorrentGetField::Id, TorrentGetField::Name, TorrentGetField::Status,
                            TorrentGetField::Percentdone, TorrentGetField::Rateupload, 
                            TorrentGetField::Totalsize, TorrentGetField::Downloaddir,
                            TorrentGetField::Addeddate]);
                let foo = client.torrent_get(fields, Some(IdOrRecent::RecentlyActive("recently-active".to_string()))).await.expect("Call Failed!").arguments.torrents;
                println!("Received {} torrents", foo.len());
                tx.send(TorrentUpdate::Partial(foo)).expect("blah");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
              }
          })
        });
        
   //     let main_context = MainContext::default();
   //     main_context.spawn_local(clone!(@strong model_ref => async move {
   //         println!("Just local spawn, useless now")
   //     }));
   }
    
    //      end of transmission code

    //model.splice(0, 0, &vector);
    //let torrent_list_model = gtk::FilterListModel::new(None, None);
    //let model = gio::ListStore::new(gtk::Label::static_type());
    
    let id_factory = gtk::SignalListItemFactory::new();

    id_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("id")
            .bind(&label, "label", gtk::Widget::NONE);
    });

    id_factory.connect_bind(move |_, _list_item| {
        let id = _list_item.property::<TorrentInfo>("item").property_value("id").get::<i64>().expect("skdfj");
        let mut xs = visible_torrents_mutex.lock().unwrap();//.push(id);
        xs.push(id);
    });


    id_factory.connect_unbind(move |_, _list_item| {
        let id = _list_item.property::<TorrentInfo>("item").property_value("id").get::<i64>().expect("skdfj");
        let mut xs = visible_torrents_mutex_rm.lock().unwrap();  
        let pos = xs.iter().position(|&x| x == id);
        match pos {
          Some(idx) => { xs.swap_remove(idx); },
          _ => ()
        }
    });

    let name_factory = gtk::SignalListItemFactory::new();
    name_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        // Bind `list_item->item->number` to `label->label`
        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("name")
            .bind(&label, "label", gtk::Widget::NONE);
    });
/*
 * 0 	Torrent is stopped
1 	Torrent is queued to verify local data
2 	Torrent is verifying local data
3 	Torrent is queued to download
4 	Torrent is downloading
5 	Torrent is queued to seed
6 	Torrent is seeding
 */
    let status_factory = gtk::SignalListItemFactory::new();
    status_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        // Bind `list_item->item->number` to `label->label`
        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("status")
            .bind(&label, "label", gtk::Widget::NONE);
    });


    let completion_factory = gtk::SignalListItemFactory::new();
    completion_factory.connect_setup(move |_, list_item| {
        let progress = gtk::ProgressBar::new();
        list_item.set_child(Some(&progress));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("percent-complete")
            .bind(&progress, "fraction", gtk::Widget::NONE);
        
        progress.set_show_text(true);
    });


    let upload_speed_factory = gtk::SignalListItemFactory::new();
    upload_speed_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        // Bind `list_item->item->number` to `label->label`
        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("rate-upload")
            .bind(&label, "label", gtk::Widget::NONE);
    });


    let total_size_factory = gtk::SignalListItemFactory::new();
    total_size_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        // Bind `list_item->item->number` to `label->label`
        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("total-size")
            .bind(&label, "label", gtk::Widget::NONE);
    });

    let download_dir_factory = gtk::SignalListItemFactory::new();
    download_dir_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        // Bind `list_item->item->number` to `label->label`
        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("download-dir")
            .bind(&label, "label", gtk::Widget::NONE);
    });


    let added_date_factory = gtk::SignalListItemFactory::new();
    added_date_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        // Bind `list_item->item->number` to `label->label`
        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("added-date")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let _filter = gtk::FilterListModel::new(Some(&model), Some(&gtk::CustomFilter::new(|x|{ x.property_value("rate-upload").get::<i64>().expect("foo") > 0 })));
    let _sorter = gtk::SortListModel::new(Some(&model), Some(&gtk::CustomSorter::new(|x,y|{
        if x.property_value("added-date").get::<i64>().expect("ad") > y.property_value("added-date").get::<i64>().expect("ad") {
            gtk::Ordering::Smaller
        } else if x.property_value("added-date").get::<i64>().expect("ad") < y.property_value("added-date").get::<i64>().expect("ad") {
            gtk::Ordering::Larger
        } else {
            gtk::Ordering::Equal
        }
    })));

    

    

    let torrent_selection_model = gtk::MultiSelection::new(Some(&_sorter));
    let torrent_list = gtk::ColumnView::new(Some(&torrent_selection_model));
    

    let c1 = gtk::ColumnViewColumn::new(Some("id"), Some(&id_factory));
    let c2 = gtk::ColumnViewColumn::new(Some("name"), Some(&name_factory));
    let c3 = gtk::ColumnViewColumn::new(Some("status"), Some(&status_factory));
    let c4 = gtk::ColumnViewColumn::new(Some("completion"), Some(&completion_factory));
    let c5 = gtk::ColumnViewColumn::new(Some("upload speed"), Some(&upload_speed_factory));
    let c6 = gtk::ColumnViewColumn::new(Some("total size"), Some(&total_size_factory));
    let c7 = gtk::ColumnViewColumn::new(Some("download dir"), Some(&download_dir_factory));
    let c8 = gtk::ColumnViewColumn::new(Some("date added"), Some(&added_date_factory));
    c1.set_resizable(true);
    c2.set_resizable(true);
    c3.set_resizable(true);
    c4.set_resizable(true);
    c5.set_resizable(true);
    c6.set_expand(true);
    c7.set_resizable(true);
    c8.set_resizable(true);
    torrent_list.append_column(&c1);
    torrent_list.append_column(&c2);
    torrent_list.append_column(&c3);
    torrent_list.append_column(&c4);
    torrent_list.append_column(&c5);
    torrent_list.append_column(&c6);
    torrent_list.append_column(&c7);
    torrent_list.append_column(&c8);

    torrent_list.set_reorderable(true);
    torrent_list.set_show_row_separators(true);
    torrent_list.set_show_column_separators(true);

    let scrolled_window = gtk::ScrolledWindow::builder()
        .min_content_width(360)
        .child(&torrent_list)
        .build();

   let status_line = gtk::Label::builder()
        .label("status bar")
        .vexpand(false)
        .build();
//    let main_view = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let bottom_pane = gtk::Paned::new(gtk::Orientation::Vertical); 
    let main_view = gtk::Paned::new(gtk::Orientation::Horizontal);

    main_view.set_start_child(&label1);
    main_view.set_end_child(&scrolled_window);
    main_view.set_resize_end_child(true);
    main_view.set_resize_start_child(false);
    main_view.set_shrink_start_child(true);

    container.append(&bottom_pane);
    container.append(&status_line);
    bottom_pane.set_start_child(&main_view);
    bottom_pane.set_end_child(&details_notebook);

    window.present();
}
//    entry.connect_search_started(clone!(@weak search_button => move |_| {
//        search_button.set_active(true);
//    }));
//
//    entry.connect_stop_search(clone!(@weak search_button => move |_| {
//        search_button.set_active(false);
//    }));
//
//    entry.connect_search_changed(clone!(@weak label => move |entry| {
//        if entry.text() != "" {
//            label.set_text(&entry.text());
//        } else {
//            label.set_text("Type to start search");
//        }
//    }));
//
//}
