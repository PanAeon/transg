mod objects;
mod utils;
mod torrent_details_grid;
mod magnet_tools;
mod notification_utils;
use crate::objects::FileObject;
use transg::transmission;
use magnet_tools::magnet_to_metainfo;
use gtk::TreeListRow;
use magnet_url::Magnet;
use notification_utils::notify;
 
use gtk::prelude::*;
use gtk::Application;
use glib::clone;
use gtk::glib;
use gtk::gio;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::env;
use lava_torrent::torrent::v1::Torrent;

        use transmission::TransmissionClient;


        enum TorrentCmd {
            AddTorrent(Option<String>, Option<String>, Option<String>, bool) // download dir, filename, metainfo, start_paused
        }


fn main() {
    gio::resources_register_include!("transgression.gresource")
        .expect("Failed to register resources.");

    let app = gtk::Application::new(
        Some("org.transgression.TransgressionOpenMagnet"),
        gio::ApplicationFlags::HANDLES_COMMAND_LINE);

    app.connect_command_line(|app, _| {
      app.activate();
      0
    });

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {


    let args: Vec<String> = env::args().collect();
    let magnet_url : String = if args.len() != 2 {
        println!("Usage: transgression-open-magnet '<magnet-url>'");
        app.quit();
        "".to_string()
    } else {
       args[1].clone() 
    };


    let window = gtk::ApplicationWindow::new(app);

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(r#"
    window {
      font-size: 14px;
      padding: 20px 20px 20px 20px;
      border-radius: 0;
	  box-shadow: none;
    }

    decoration {
	box-shadow: none;
}

decoration:backdrop {
	box-shadow: none;
}

    .sidebar-category-name {
      font-size: 14px;
    }
    .sidebar-category-count {
      font-size: 14px;
      margin-right: 12px;
    }
    .details-label {
      font-weight: bold;
    }
    .simple-dialog > .dialog-vbox {
      padding: 10px 20px 20px 10px;
      background-color: @theme_bg_color;
    }
    .sidebar > row {
        margin-top: 4px;
        margin-left: 8px;
        margin-right: 16px;
    }
/*    progressbar.horizontal > trough, progress {
      min-width: 40px;
    } */
     "#.as_bytes());
   gtk::StyleContext::add_provider_for_display(
     &gtk::gdk::Display::default().unwrap(),
     &css_provider,
     gtk::STYLE_PROVIDER_PRIORITY_USER
     );







        let client = TransmissionClient::new(&"http://192.168.1.217:9091/transmission/rpc".to_string());
        let (sender, rx2) = mpsc::channel(); 


        thread::spawn(move || {
          use tokio::runtime::Runtime;
          let rt = Runtime::new().expect("create tokio runtime");


          
          loop {
            let cmd = rx2.recv().expect("probably ticker thread panicked");

          rt.block_on(async {
            match cmd {
                TorrentCmd::AddTorrent(download_dir, filename, metainfo, paused) => {
                    let tadd = transmission::TorrentAdd {
                        cookies: None,
                        bandwith_priority: None,
                        download_dir,
                        filename,
                        metainfo,
                        files_unwanted: None,
                        files_wanted: None,
                        labels: None,
                        paused: Some(paused),
                        peer_limit: None,
                        priority_high: None,
                        priority_low: None,
                        priority_normal: None
                    };
                    println!("adding torrent");
                    let res = client.torrent_add(&tadd).await.expect("ooph5");
                    let result = res.as_object().expect("should return object").get("result").expect("must result").as_str().unwrap().to_string();
                    if result == "success" {
                        let _ =  notify("Torrent Added!", "").await; // TODO: add name
                    } else {
                        let _ = notify("Error!", "").await;
                      println!("{:?}", res);
                    }
                } 
            }
          });

                  
          }       
          });
   

//    let _window = Rc::new(window);
 
   
 //   let sender = tx2.clone();


    let model = gtk::StringList::new(&vec![
                                     "/var/lib/transmission/Downloads/films",
                                     "/var/lib/transmission/Downloads/games",
                                     "/var/lib/transmission/Downloads/music",
                                     "/var/lib/transmission/Downloads/lessons",
                                     "/var/lib/transmission/Downloads/blues",
    ]);

  let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);

  let file_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  vbox.append(&file_hbox);

  let l = gtk::Label::new(Some("Torrent file"));
  file_hbox.append(&l);

  let _cancel_fetch = Rc::new(Cell::new(false));
  let _magnet_done = Rc::new(Cell::new(false));
  let file_chooser = gtk::Button::new();
  file_hbox.append(&file_chooser);


    window.set_title(Some("Transgression Open Magnet"));
  //dialog.add_button("Cancel", gtk::ResponseType::Cancel);
  //dialog.add_button("Add", gtk::ResponseType::Ok);

  let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  let l = gtk::Label::new(Some("Destination"));
  hbox.append(&l);
  let destination = gtk::DropDown::builder().model(&model).build();
  hbox.append(&destination);
  vbox.append(&hbox);

  let file_table = gtk::ColumnView::new(None::<&gtk::NoSelection>);
  let scrolled_files = build_bottom_files(&file_table, false);


  let progressbar = gtk::ProgressBar::new();
  let _progressbar = Rc::new(progressbar);
 // let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&files))));
  let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&vec![]))));
  let files_selection = gtk::NoSelection::new(Some(&stupid_model));
  file_table.set_model(Some(&files_selection));


  let magnet_data =  Rc::new(RefCell::new(None));
  let _file_table = Rc::new(file_table);

  let foo : String = magnet_url.chars().into_iter().take(60).collect();
  if let Result::Ok(m) = Magnet::new(&magnet_url) { 
          let s = m.dn.as_ref().map(|l| urlencoding::decode(l).expect("UTF-8").into_owned());
//        println!("{}", m.dn.expect("foo"));
        file_chooser.set_label(&s.unwrap_or(foo));
  gtk::glib::MainContext::default().spawn_local(pulse_progress(Rc::clone(&_progressbar), Rc::clone(&_magnet_done)));
  gtk::glib::MainContext::default().spawn_local(fetch_magnet_link(magnet_url.to_string(), Rc::clone(&_file_table), Rc::clone(&_cancel_fetch), Rc::clone(&_magnet_done), Rc::clone(&magnet_data)));
  } 
  //file_chooser.set_label(&foo);

  scrolled_files.set_min_content_width(580);
  scrolled_files.set_min_content_height(320);
  vbox.append(&scrolled_files);
  vbox.append(&*_progressbar);

  let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
//  let l = gtk::Label::new(Some("Torrent priority"));
//  hbox.append(&l);
//  let priority = gtk::DropDown::builder().build();
//  hbox.append(&priority);
  vbox.append(&hbox);

  let start_paused_checkbox = gtk::CheckButton::builder().active(false).label("Start paused").build(); 
  vbox.append(&start_paused_checkbox);
  let delete_torrent_file = gtk::CheckButton::builder().active(true).label("Delete torrent file").build(); 
  vbox.append(&delete_torrent_file);
  delete_torrent_file.set_sensitive(false);

  let bottom_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  bottom_box.set_halign(gtk::Align::End);
  vbox.append(&bottom_box);
  let ok_button = gtk::Button::new();
  ok_button.set_label("Ok");
  bottom_box.append(&ok_button);
  let cancel_button = gtk::Button::new();
  cancel_button.set_label("Cancel");
  bottom_box.append(&cancel_button);

  window.set_child(Some(&vbox)); 

  cancel_button.connect_clicked(clone!(@weak app => move |_| {
      app.quit();
  }));
  ok_button.connect_clicked(clone!(@weak app => move |_| {
     let start_paused =start_paused_checkbox.is_active();
     let folder = destination.selected_item().map(|x| x.property_value("string").get::<String>().expect("should be string")); 
     _cancel_fetch.set(true); 
     if let Some(data) = &*magnet_data.borrow() {
         let metainfo = base64::encode(data);
         sender.send(TorrentCmd::AddTorrent(folder, None, Some(metainfo), start_paused)).expect("failure snd move");
       } else { 
         sender.send(TorrentCmd::AddTorrent(folder, Some(magnet_url.to_string()), None, start_paused)).expect("failure snd move");
       }
     thread::sleep(std::time::Duration::from_millis(300));
     app.quit();
  }));

  window.present();
}

fn get_children(tree: &Vec<utils::Node>, path: String) -> Vec<utils::Node> {
  fn do_get_children(tree: &Vec<utils::Node>, mut path: Vec<&str>) -> Vec<utils::Node> {
  for n in tree {
    if n.name == path[0] {
        if path.len() == 1 {
            return n.children.clone();
        } else {
            path.remove(0);
            return do_get_children(&n.children, path);
        }
    }
  }
  vec![]
  }
  do_get_children(tree, path.split('/').collect())
}

fn create_file_model(tree: &Rc<RefCell<Vec<utils::Node>>>)  ->  gtk::TreeListModel 
  {

    let m0 = gio::ListStore::new(FileObject::static_type());
    let mut v0 : Vec<FileObject> = vec![];
    for node in tree.borrow().iter() { 
          let fraction: f64 = if node.size == 0 { 0.0 } else { node.downloaded as f64 / node.size as f64 };
          v0.push(FileObject::new(&node.name.clone(), &node.path.clone(), &node.size.clone(), &fraction, true, 3)); 
    }
    m0.splice(0, 0, &v0);

    let tree = tree.clone(); // TODO: probably no need, just use clone! with strong reference..

    let tree_fun =  move |x:&glib::Object|{
          let path = x.property_value("path").get::<String>().unwrap();
          let children = get_children(&tree.borrow(), path);
          if children.len() > 0 {
            let m0 = gio::ListStore::new(FileObject::static_type());
            let mut v0 : Vec<FileObject> = vec![];
            for node in &children { 
              let fraction: f64 = if node.size == 0 { 0.0 } else { node.downloaded as f64 / node.size as f64 };
              v0.push(FileObject::new(&node.name.clone(), &node.path.clone(), &node.size.clone(), &fraction, true, 3)); 
            }
            m0.splice(0, 0, &v0);
            Some(m0.upcast())
          } else {
              None
          }
        };

    let model = gtk::TreeListModel::new(&m0, false, true, tree_fun);
    
    model
}





fn build_bottom_files(file_table: &gtk::ColumnView, include_progress: bool) -> gtk::ScrolledWindow {

    // TODO: realize consequences of your findings...
    let exp_factory = gtk::SignalListItemFactory::new();
    exp_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
         
        let expander = gtk::TreeExpander::new();
        list_item.set_child(Some(&expander));
        
        expander.set_child(Some(&label));
        
        list_item
            .property_expression("item")
            .bind(&expander, "list-row", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<TreeListRow>("item")
            .chain_property::<FileObject>("name")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    file_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Name")
          .expand(true)
          .factory(&exp_factory)
          .build()
        );
    

    let size_factory = gtk::SignalListItemFactory::new();
    size_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);

        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TreeListRow>("item")
            .chain_property::<FileObject>("size")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                utils::format_size(i.try_into().unwrap())
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    file_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Size")
          .expand(true)
          .factory(&size_factory)
          .build()
        );

    if include_progress {
    let progress_factory = gtk::SignalListItemFactory::new();
    progress_factory.connect_setup(move |_, list_item| {
        let progress = gtk::ProgressBar::new();
        list_item.set_child(Some(&progress));

        list_item
            .property_expression("item")
            .chain_property::<TreeListRow>("item")
            .chain_property::<FileObject>("progress")
            .bind(&progress, "fraction", gtk::Widget::NONE);
        
        progress.set_show_text(true);
    });
     
    file_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Progress")
          .expand(true)
          .factory(&progress_factory)
          .build()
        );
    }

    let download_factory = gtk::SignalListItemFactory::new();
    download_factory.connect_setup(move |_, list_item| {
        let checkbox = gtk::CheckButton::new();
        list_item.set_child(Some(&checkbox));
        checkbox.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TreeListRow>("item")
            .chain_property::<FileObject>("download")
            .bind(&checkbox, "active", gtk::Widget::NONE);
    });
    
    file_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Download")
          .expand(true)
          .factory(&download_factory)
          .build()
        );

    let priority_factory = gtk::SignalListItemFactory::new();
    priority_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TreeListRow>("item")
            .chain_property::<FileObject>("priority")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, priority: i8| {
              match priority {
                -1 => "Low",
                0  => "Normal",
                1  => "High",
                _ =>  "Normal"
              }
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    file_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Priority")
          .expand(true)
          .factory(&priority_factory)
          .build()
        );

    gtk::ScrolledWindow::builder()
        .min_content_width(360)
        .vexpand(true)
        .child(file_table)
        .build()
}


async fn fetch_magnet_link(uri: String, _file_table: Rc<gtk::ColumnView>, cancellation: Rc<Cell<bool>>, done: Rc<Cell<bool>>, magnet_data: Rc<RefCell<Option<Vec<u8>>>>) {
    let maybe_torrent = magnet_to_metainfo(&uri, cancellation).await; 
    done.set(true);
    if let Some(ref data) = maybe_torrent {
      let res = Torrent::read_from_bytes(data);
      if let Result::Ok(torrent) = res {
        if let Some(xs) = torrent.files { 
          let files = xs.iter().map(|f| transmission::File {name: f.path.as_path().to_str().expect("bloody name?").to_string(), length: f.length as u64, bytes_completed: 0  } ).collect();
          let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&files))));
          let files_selection = gtk::NoSelection::new(Some(&stupid_model));
          _file_table.set_model(Some(&files_selection));
          (*magnet_data).replace(Some(data.to_vec()));
        } else {
            // the file is the name 
            // FIXME: prevent interactions
            // TODO: same needs to be done for file details
          let files =  vec![transmission::File {name: torrent.name, length: torrent.length as u64, bytes_completed: 0  }];
          let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&files))));
          let files_selection = gtk::NoSelection::new(Some(&stupid_model));
          _file_table.set_model(Some(&files_selection));
          _file_table.set_sensitive(false);
          (*magnet_data).replace(Some(data.to_vec()));
        }
      } else {
          println!("Error: {:?}", res); // FIXME: error mngmnt
      }
    }
}

async fn pulse_progress(_progressbar: Rc<gtk::ProgressBar>, _magnet_done: Rc<Cell<bool>>) {
    loop {
        if _magnet_done.get() {
            break;
        }
        _progressbar.pulse();
       async_std::task::sleep(std::time::Duration::from_millis(200)).await;
    }
    _progressbar.set_fraction(1.0);
}
