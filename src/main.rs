mod objects;
mod utils;
mod torrent_details_grid;
mod magnet_tools;
mod notification_utils;
mod command_processor;
mod torrent_stats;
mod file_table;
mod create_torrent_dialog;
use crate::objects::{ TorrentInfo, TorrentDetailsObject, Stats, PeerObject, TrackerObject, FileObject, CategoryObject};
use crate::torrent_details_grid::TorrentDetailsGrid;
use transg::transmission;
use crate::file_table::{create_file_model, build_bottom_files};
use crate::create_torrent_dialog::add_torrent_dialog;
 
use gtk::prelude::*;
use gtk::Application;
use glib::clone;
use gtk::glib;
use gtk::gio;
use utils::{update_torrent_details, format_time, json_value_to_torrent_info};
//use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::rc::Rc;
//use std::sync::mpsc::{Sender, Receiver};
use tokio::sync::mpsc;
use command_processor::{TorrentCmd, TorrentUpdate, CommandProcessor};
use crate::torrent_stats::update_torrent_stats;

// FIXME: remove this duplication
pub const STOPPED: i64 = 0;
pub const VERIFY_QUEUED: i64 = 1;
pub const VERIFYING: i64 = 2;
pub const DOWN_QUEUED: i64 = 3;
pub const DOWNLOADING: i64 = 4;
pub const SEED_QUEUED: i64 = 5;
pub const SEEDING: i64 = 6;
pub const ALL: i64 = -1;
pub const SEPARATOR: i64 = -2;
pub const FOLDER: i64 = -3;
pub const ERROR: i64 = -4;

const NONE_EXPRESSION: Option<&gtk::Expression> = None;


fn main() {
    gio::resources_register_include!("transgression.gresource")
        .expect("Failed to register resources.");

    let app = gtk::Application::new(
        Some("org.transgression.Transgression"),
        Default::default());

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {

    let  model = gio::ListStore::new(TorrentInfo::static_type());
    let category_model = gio::ListStore::new(CategoryObject::static_type());
    let stats = Stats::new(0, 0, 0);
    category_model.splice(0, 0, &vec![
        CategoryObject::new("All".to_string(), 0, ALL,  false, "".to_string()),
        CategoryObject::new("Paused".to_string(), 0, STOPPED,  false, "".to_string()),
        CategoryObject::new("Verification queued".to_string(), 0, VERIFY_QUEUED,  false, "".to_string()),
        CategoryObject::new("Checking".to_string(), 0,VERIFYING,  false, "".to_string()),
        CategoryObject::new("Download queued".to_string(), 0, DOWN_QUEUED, false, "".to_string()),
        CategoryObject::new("Downloading".to_string(), 0, DOWNLOADING, false, "".to_string()),
        CategoryObject::new("Seeding queued".to_string(), 0, SEED_QUEUED, false, "".to_string()),
        CategoryObject::new("Seeding".to_string(), 0, SEEDING, false, "".to_string()),
        CategoryObject::new("Error".to_string(), 0, ERROR, false, "".to_string()),
        CategoryObject::new("-".to_string(), 0, SEPARATOR, false, "".to_string()),
    ]);

    let details_object = TorrentDetailsObject::new(
        &u64::MAX, &"".to_string(), &0, &0, &0, &0, &0, &"".to_string(), &"".to_string(), 
        &"".to_string(), &0, &0, &0.0, &0, &0, &0, &0.0, 
        &0, &0, &0, &0, &"".to_string(), &0, &"".to_string()
        );

    let peers_model = gio::ListStore::new(PeerObject::static_type());
    let tracker_model = gio::ListStore::new(TrackerObject::static_type());
    
    let file_table = gtk::ColumnView::new(None::<&gtk::NoSelection>);

    let window = gtk::ApplicationWindow::new(app);
    window.set_default_size(1920, 1080);
    window.set_title(Some("Transgression"));

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(r#"
    window {
      font-size: 14px;
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


    let header_bar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&header_bar));


//    let menu_button = gtk::MenuButton::new();
//    menu_button.set_icon_name("open-menu-symbolic");
    let settings_button = gtk::ToggleButton::new();
    settings_button.set_icon_name("emblem-system-symbolic");
    header_bar.pack_end(&settings_button);


    //let menu_builder = gtk::Builder::from_resource("/org/transgression/main_menu.ui");
    //let menu_model = menu_builder.object::<gio::MenuModel>("menu").expect("can't find menu");
    //menu_button.set_menu_model(Some(&menu_model));
    //menu_button.set_primary(true);

    let search_button = gtk::ToggleButton::new();
    search_button.set_icon_name("system-search-symbolic");
    header_bar.pack_end(&search_button);


    let sort_button = gtk::Button::new();
    sort_button.set_icon_name("view-list-symbolic");
    header_bar.pack_end(&sort_button);

    let sort_popover = gtk::Popover::new();
    let sort_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let sort_label = gtk::Label::new(Some("Sort by"));
    let sort_by_date_added = gtk::CheckButton::builder().label("Date added").build();
    let sort_by_name = gtk::CheckButton::builder().label("Name").group(&sort_by_date_added).build();
    let sort_by_size = gtk::CheckButton::builder().label("Size").group(&sort_by_date_added).build();
    let sort_by_uploaded = gtk::CheckButton::builder().label("Uploaded bytes").group(&sort_by_date_added).build();
    let sort_by_ratio = gtk::CheckButton::builder().label("Ratio").group(&sort_by_date_added).build();
    let sort_by_upload_speed = gtk::CheckButton::builder().label("Upload speed").group(&sort_by_date_added).build();

    let date_added_sorter = sort_by_property::<i64>("added-date", true); 
    let _sorter = gtk::SortListModel::new(Some(&model), Some(&date_added_sorter));

    sort_by_date_added.connect_toggled(clone!(@weak _sorter => move |_|{
      _sorter.set_sorter(Some(&date_added_sorter));
    }));
    let name_sorter = sort_by_property::<String>("name", false);

    sort_by_name.connect_toggled(clone!(@weak _sorter => move |_|{
      _sorter.set_sorter(Some(&name_sorter));
    }));

    let size_sorter = sort_by_property::<i64>("size_when_done", true);

    sort_by_size.connect_toggled(clone!(@weak _sorter => move |_|{
      _sorter.set_sorter(Some(&size_sorter));
    }));
    let uploaded_sorter = sort_by_property::<i64>("uploaded_ever", true);

    sort_by_uploaded.connect_toggled(clone!(@weak _sorter => move |_|{
      _sorter.set_sorter(Some(&uploaded_sorter));
    }));

    let ratio_sorter = sort_by_property::<f64>("upload_ratio", true);

    sort_by_ratio.connect_toggled(clone!(@weak _sorter => move |_|{
      _sorter.set_sorter(Some(&ratio_sorter));
    }));
    let upload_speed_sorter = sort_by_property::<i64>("rate_upload", true); 

    sort_by_upload_speed.connect_toggled(clone!(@weak _sorter => move |_|{
      _sorter.set_sorter(Some(&upload_speed_sorter));
    }));

    sort_box.append(&sort_label);
    sort_box.append(&sort_by_date_added);
    sort_box.append(&sort_by_name);
    sort_box.append(&sort_by_size);
    sort_box.append(&sort_by_uploaded);
    sort_box.append(&sort_by_ratio);
    sort_box.append(&sort_by_upload_speed);
    sort_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let reverse_sort_order = gtk::CheckButton::builder().label("Reverse sort order").build();
    sort_box.append(&reverse_sort_order);
    sort_popover.set_child(Some(&sort_box));
    //sort_popover.set_pointing_to(&sort_button);
    sort_popover.set_parent(&sort_button);
    sort_popover.set_position(gtk::PositionType::Bottom);

    sort_by_date_added.set_active(true);
    sort_button.connect_clicked(clone!( @weak sort_popover => move |_| {
       sort_popover.popup();
    }));

    let add_button = gtk::Button::new();
    add_button.set_icon_name("list-add");
    header_bar.pack_start(&add_button);

    let add_magnet_button = gtk::Button::new();
    add_magnet_button.set_icon_name("emblem-shared-symbolic");
    header_bar.pack_start(&add_magnet_button);


    let start_button = gtk::Button::new();
    start_button.set_icon_name("media-playback-start");
    header_bar.pack_start(&start_button);
    start_button.set_action_name(Some("win.torrent-start"));


    let pause_button = gtk::Button::new();
    pause_button.set_icon_name("media-playback-pause");
    header_bar.pack_start(&pause_button);
    pause_button.set_action_name(Some("win.torrent-stop"));

    let queue_up_button = gtk::Button::new();
    queue_up_button.set_icon_name("go-up");
    header_bar.pack_start(&queue_up_button);
    queue_up_button.set_action_name(Some("win.queue-up"));

    let queue_down_button = gtk::Button::new();
    queue_down_button.set_icon_name("go-down");
    header_bar.pack_start(&queue_down_button);
    queue_down_button.set_action_name(Some("win.queue-down"));

    let remove_button = gtk::Button::new();
    remove_button.set_icon_name("list-remove");
    header_bar.pack_start(&remove_button);
    remove_button.set_action_name(Some("win.torrent-remove"));

    let remove_with_files_button = gtk::Button::new();
    remove_with_files_button.set_icon_name("edit-delete");
    header_bar.pack_start(&remove_with_files_button);
    remove_with_files_button.set_action_name(Some("win.torrent-remove-with-data"));

    let topstack = gtk::Stack::new();
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    topstack.add_named(&container, Some("main"));
    let settings_hbox = gtk::Box::new(gtk::Orientation::Horizontal,0);
    let settings_sidebar = gtk::StackSidebar::new();
    settings_hbox.append(&settings_sidebar);
    let settings_stack = gtk::Stack::new();
    settings_stack.set_transition_type(gtk::StackTransitionType::SlideUpDown);
    settings_sidebar.set_stack(&settings_stack);


    let l1 = gtk::Label::new(Some("l1"));
    settings_stack.add_titled(&l1, None, "Connections");
    let l2 = gtk::Label::new(Some("l1"));
    settings_stack.add_titled(&l2, None, "General");
    let l3 = gtk::Label::new(Some("l1"));
    settings_stack.add_titled(&l3, None, "Actions");
    let l4 = gtk::Label::new(Some("l2"));
    settings_stack.add_titled(&l4, None, "Folders");

    settings_hbox.append(&settings_stack);
    topstack.add_named(&settings_hbox, Some("settings"));
    topstack.set_visible_child_name("main");
    window.set_child(Some(&topstack));
    
    settings_button.connect_toggled(clone!(@weak topstack => move |button| {
      if button.is_active() {
         topstack.set_visible_child_name("settings");
      } else {
         topstack.set_visible_child_name("main");
      }
    }));


     let (mut processor, rx) = CommandProcessor::create();
     let tx2 = processor.get_sender();


     rx.attach(None, clone!(@weak model, @weak details_object, @weak peers_model, @weak tracker_model, @weak file_table,  @weak category_model, @weak stats => @default-return Continue(false), move |update:TorrentUpdate|{
          match update {
            TorrentUpdate::Full(xs) => {
              let mut torrents: Vec<TorrentInfo> = xs.as_array().unwrap()
              .iter()
              .skip(1)
              .map(|it| {
                  json_value_to_torrent_info(it)
              }).collect();
              torrents.sort_by(|a, b| a.property_value("id").get::<i64>().expect("fkjf").cmp(&b.property_value("id").get::<i64>().expect("xx")));
              model.remove_all();
              model.splice(0, 0, &torrents);
              update_torrent_stats(&model, &category_model );
                // mutex is not needed as we do this in the ui thread 
                // *group_stats_mutex.lock().unwrap() = group_stats;
              },
           TorrentUpdate::Partial(xs, removed, update_count) => {
//                  println!(">>update received, num torrents: {} ", xs.as_array().unwrap().len()); 
                  let removed: Vec<i64> = removed.as_array().unwrap().iter().map(|x| x.as_i64().unwrap()).collect();
                  if removed.len() > 0 {
                      let mut i = 0;
                      while let Some(y) = model.item(i) {
                        let id = y.property_value("id").get::<i64>().expect("skdfj");
                        if removed.contains(&id) {
                          model.remove(i);
                        } else {
                            i += 1;
                        }
                      }
                  } 
                  let mut xxs = xs.as_array().unwrap().clone(); 
                  xxs.remove(0);
                  xxs.sort_by(|a, b| a.as_array().unwrap()[0].as_i64().unwrap().cmp(&b.as_array().unwrap()[0].as_i64().unwrap()));

                  let mut i = 0;
                  for x in xxs.iter() {
                      let ys = x.as_array().unwrap();
                      while let Some(y) = model.item(i) {
                        if ys[0].as_i64().unwrap() == y.property_value("id").get::<i64>().expect("skdfj") {
                          y.set_property("status", ys[2].as_i64().unwrap().to_value());
                          y.set_property("percent-done", ys[3].as_f64().unwrap().to_value());
                          y.set_property("error", ys[4].as_i64().unwrap().to_value());
                          y.set_property("error-string", ys[5].as_str().unwrap().to_value());
                          y.set_property("eta", ys[6].as_i64().unwrap().to_value());
                          y.set_property("queue-position", ys[7].as_i64().unwrap().to_value());
                          y.set_property("is-finished", ys[8].as_bool().unwrap().to_value());
                          y.set_property("is-stalled", ys[9].as_bool().unwrap().to_value());
                          y.set_property("metadata-percent-complete", ys[10].as_f64().unwrap().to_value());
                          y.set_property("peers-connected", ys[11].as_i64().unwrap().to_value());
                          y.set_property("rate-download", ys[12].as_i64().unwrap().to_value());
                          y.set_property("rate-upload", ys[13].as_i64().unwrap().to_value());
                          y.set_property("recheck-progress", ys[14].as_f64().unwrap().to_value());
                          y.set_property("size-when-done", ys[15].as_i64().unwrap().to_value());
                          y.set_property("download-dir", ys[16].as_str().unwrap().to_value());
                          y.set_property("uploaded-ever", ys[17].as_i64().unwrap().to_value());
                          y.set_property("upload-ratio", ys[18].as_f64().unwrap().to_value());
                          y.set_property("added-date", ys[19].as_i64().unwrap().to_value());
                          break;
                        } else if ys[0].as_i64().unwrap() < y.property_value("id").get::<i64>().expect("skdfj") {
                            model.insert(i, &json_value_to_torrent_info(x));
                            break; // TODO: please, test this..
                        }
                        i+=1;
                      }
                      if model.item(i).is_none() {
                        model.append(&json_value_to_torrent_info(x));      
                      }
                  } 
                  if update_count % 4 == 0 {
                     update_torrent_stats(&model, &category_model);
                  }
                  },
                  TorrentUpdate::Stats(s) => {
                    stats.set_property("upload", s.upload_speed.to_value());
                    stats.set_property("download", s.download_speed.to_value());
                  },
                  TorrentUpdate::FreeSpace(s) => {
                    stats.set_property("free-space", s.size_bytes.to_value());
                  },
                  TorrentUpdate::Details(details) => {
            let previous_id = details_object.property_value("id").get::<u64>().unwrap();
            update_torrent_details(details_object, &details);

            if previous_id != details.id {
              let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&details.files))));
              let files_selection = gtk::NoSelection::new(Some(&stupid_model));
              file_table.set_model(Some(&files_selection));
            }
            let trackers: Vec<TrackerObject> = details.trackers.iter().zip(details.tracker_stats.iter())
                .map(|(t,ts)| TrackerObject::new(&t.id, &t.announce,
                                                 &t.tier, &ts.leecher_count,
                                                 &ts.host, &t.scrape, &ts.seeder_count,
                                                 &ts.last_announce_peer_count,
                                                 &ts.last_announce_result,
                                                 &ts.last_announce_time))
                .collect();
            tracker_model.remove_all();
            tracker_model.splice(0, 0, &trackers);

            if previous_id != details.id {
              let peers: Vec<PeerObject> = details.peers.iter().map(|p| PeerObject::new(&p.address, &p.client_name, &p.progress, &p.rate_to_client, &p.rate_to_peer, &p.flag_str)).collect();
              peers_model.remove_all();
              peers_model.splice(0, 0, &peers);
            } else {
                // TODO: maybe merge peers?
              let peers: Vec<PeerObject> = details.peers.iter().map(|p| PeerObject::new(&p.address, &p.client_name, &p.progress, &p.rate_to_client, &p.rate_to_peer, &p.flag_str)).collect();
              peers_model.remove_all();
              peers_model.splice(0, 0, &peers);
            }

                  }
              }
          }

           Continue(true)
        ));

     processor.run("http://192.168.1.217:9091/transmission/rpc");


    let name_factory = gtk::SignalListItemFactory::new();
    name_factory.connect_setup(move |_, list_item| {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&hbox));
        label.set_halign(gtk::Align::Start);
        let icon = gtk::Image::new();
        let error_icon = gtk::Image::new();
        error_icon.set_icon_name(Some("dialog-error-symbolic"));
        hbox.append(&icon);
        hbox.append(&error_icon);
        hbox.append(&label);

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("error") // FIXME: now I need also to merge error here somehow
            .chain_closure::<bool>(gtk::glib::closure!(|_: Option<glib::Object>, error: i64| {
                error > 0
            }))
            .bind(&error_icon, "visible", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("error") // FIXME: now I need also to merge error here somehow
            .chain_closure::<bool>(gtk::glib::closure!(|_: Option<glib::Object>, error: i64| {
                error == 0
            }))
            .bind(&icon, "visible", gtk::Widget::NONE);
        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("status") // FIXME: now I need also to merge error here somehow
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, status: i64| {
              match status {
                STOPPED => "media-playback-stop",
                VERIFY_QUEUED => "view-refresh",
                VERIFYING => "view-refresh",
                DOWN_QUEUED => "network-receive",
                DOWNLOADING => "arrow-down-symbolic",
                SEED_QUEUED => "network-transmit",
                SEEDING => "arrow-up-symbolic",
                _ => "dialog-question"
              }
            }))
            .bind(&icon, "icon-name", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("name")
            .bind(&label, "label", gtk::Widget::NONE);
    });

   // let status_factory = gtk::SignalListItemFactory::new();
   // status_factory.connect_setup(label_setup::<TorrentInfo, i64, _>("status", |x| format!("{}", x)));

    let eta_factory = gtk::SignalListItemFactory::new();
    eta_factory.connect_setup(label_setup::<TorrentInfo, i64, _>("eta", utils::format_eta));

    let ratio_factory = gtk::SignalListItemFactory::new();
    ratio_factory.connect_setup(label_setup::<TorrentInfo, f64, _>("upload-ratio", |x| format!("{:.2}", x)));

    let num_peers_factory = gtk::SignalListItemFactory::new();
    num_peers_factory.connect_setup(label_setup::<TorrentInfo, i64, _>("peers-connected", |x| if x > 0 {format!("{}", x)} else {"".to_string()}));

    let completion_factory = gtk::SignalListItemFactory::new();
    completion_factory.connect_setup(move |_, list_item| {
        let progress = gtk::ProgressBar::new();
        list_item.set_child(Some(&progress));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("percent-done")
            .bind(&progress, "fraction", gtk::Widget::NONE);
        
        progress.set_show_text(true);
    });
    
    let download_speed_factory = gtk::SignalListItemFactory::new(); // TODO: generalize
    download_speed_factory.connect_setup(label_setup::<TorrentInfo, i64, _>("rate-download", utils::format_download_speed));


    let upload_speed_factory = gtk::SignalListItemFactory::new();
    upload_speed_factory.connect_setup(label_setup::<TorrentInfo, i64, _>("rate-upload", utils::format_download_speed));


    let total_size_factory = gtk::SignalListItemFactory::new();
    total_size_factory.connect_setup(label_setup::<TorrentInfo, i64, _>("size-when-done", utils::format_size));

    let uploaded_ever_factory = gtk::SignalListItemFactory::new();
    uploaded_ever_factory.connect_setup(label_setup::<TorrentInfo, i64, _>("uploaded-ever", utils::format_size));

    let name_expr = gtk::PropertyExpression::new(TorrentInfo::static_type(), NONE_EXPRESSION, "name");
    let name_filter = gtk::StringFilter::new(Some(name_expr));
    name_filter.set_match_mode(gtk::StringFilterMatchMode::Substring);

    let category_filter    = gtk::CustomFilter::new(move |_| true);

    let main_filter = gtk::EveryFilter::new();
    main_filter.append(&name_filter);
    main_filter.append(&category_filter);
    let name_filter_model = gtk::FilterListModel::new(Some(&_sorter), Some(&main_filter));
    name_filter_model.set_incremental(false);
    
    let torrent_selection_model = gtk::MultiSelection::new(Some(&name_filter_model));
    let sender = tx2.clone();
torrent_selection_model.connect_selection_changed(move |_model, _pos, _num_items| {
    let last = _pos + _num_items;
    let mut i = _pos;
    while i <= last {
      if _model.is_selected(i) {
          sender.blocking_send(TorrentCmd::GetDetails(_model.item(i).unwrap().property_value("id").get::<i64>().unwrap())).expect("can't send details");
        break;
      }
      i += 1;
    } 
});

    let torrent_list = gtk::ColumnView::new(Some(&torrent_selection_model));
    

    let name_col = gtk::ColumnViewColumn::new(Some("Name"), Some(&name_factory));
    let completion_col = gtk::ColumnViewColumn::new(Some("Completion"), Some(&completion_factory));
    let eta_col = gtk::ColumnViewColumn::new(Some("Eta      "), Some(&eta_factory));
    let num_peers_col = gtk::ColumnViewColumn::new(Some("Peers  "), Some(&num_peers_factory));
    let download_speed_col = gtk::ColumnViewColumn::new(Some("Download     "), Some(&download_speed_factory));
    let upload_speed_col = gtk::ColumnViewColumn::new(Some("Upload      "), Some(&upload_speed_factory));
    let size_col = gtk::ColumnViewColumn::new(Some("Size           "), Some(&total_size_factory));
    let uploaded_ever_col = gtk::ColumnViewColumn::new(Some("Uploaded     "), Some(&uploaded_ever_factory));
    let ratio_col = gtk::ColumnViewColumn::new(Some("Ratio    "), Some(&ratio_factory));

    name_col.set_resizable(true);
    name_col.set_expand(true);

    completion_col.set_resizable(true);
//    eta_col.set_resizable(true);
    eta_col.set_fixed_width(75);
    download_speed_col.set_resizable(true);
    upload_speed_col.set_resizable(true);
    size_col.set_resizable(true);
    eta_col.set_resizable(true);
    ratio_col.set_resizable(true);
    uploaded_ever_col.set_resizable(true);

    torrent_list.append_column(&name_col);
    torrent_list.append_column(&completion_col);
    torrent_list.append_column(&eta_col);
    torrent_list.append_column(&download_speed_col);
    torrent_list.append_column(&upload_speed_col);
    torrent_list.append_column(&num_peers_col);
    torrent_list.append_column(&size_col);
    torrent_list.append_column(&ratio_col);
    torrent_list.append_column(&uploaded_ever_col);

    torrent_list.set_reorderable(true);
    torrent_list.set_show_row_separators(false);
    torrent_list.set_show_column_separators(false);

    let search_bar = gtk::SearchBar::builder()
        .valign(gtk::Align::Start)
        .key_capture_widget(&window)
        .build();

    search_button
        .bind_property("active", &search_bar, "search-mode-enabled")
        .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
        .build();

    let entry = gtk::SearchEntry::new();
    entry.set_hexpand(true);
    search_bar.set_child(Some(&entry));

    let scrolled_window = gtk::ScrolledWindow::builder()
        .min_content_width(360)
        .vexpand(true)
        .child(&torrent_list)
        .build();

    let right_hbox = gtk::Box::new(gtk::Orientation::Vertical, 3);
    right_hbox.append(&search_bar);
    right_hbox.append(&scrolled_window);

   let status_line = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).halign(gtk::Align::End).valign(gtk::Align::End).margin_end(20).build(); 
   
   let upload_line_label = gtk::Label::builder().label("Upload: ").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   upload_line_label.set_margin_top(5);
   upload_line_label.set_margin_bottom(5);
   
   let upload_value_label = gtk::Label::builder().label("").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   stats.property_expression("upload")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                utils::format_download_speed(i.try_into().unwrap())
            }))
       .bind(&upload_value_label, "label", gtk::Widget::NONE);
   let download_line_label = gtk::Label::builder().label(" Download: ").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   let download_value_label = gtk::Label::builder().label("").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   stats.property_expression("download")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                utils::format_download_speed(i.try_into().unwrap())
            }))
       .bind(&download_value_label, "label", gtk::Widget::NONE);
   let free_space_label = gtk::Label::builder().label(" Free space: ").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   let free_space_value_label = gtk::Label::builder().label("").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   stats.property_expression("free-space")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                utils::format_size(i.try_into().unwrap())
            }))
       .bind(&free_space_value_label, "label", gtk::Widget::NONE);
   status_line.append(&upload_line_label);
   status_line.append(&upload_value_label);
   status_line.append(&download_line_label);
   status_line.append(&download_value_label);
   status_line.append(&free_space_label);
   status_line.append(&free_space_value_label);

//    let main_view = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let bottom_pane = gtk::Paned::new(gtk::Orientation::Vertical); 
    let main_view = gtk::Paned::new(gtk::Orientation::Horizontal);


    // left pane =================================================================

    let left_pane = gtk::Box::builder().vexpand(true).orientation(gtk::Orientation::Vertical).build();

    let category_factory = gtk::SignalListItemFactory::new();

    category_factory.connect_setup(move |_, list_item| {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        list_item.set_child(Some(&hbox));

        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
        separator.set_hexpand(true);
        separator.set_sensitive(false);
        separator.set_margin_top(8);
        separator.set_margin_bottom(8);

        let name_label = gtk::Label::new(None);
        name_label.set_css_classes(&["sidebar-category-name"]);
        let count_label = gtk::Label::new(None);
        count_label.set_css_classes(&["sidebar-category-count"]);
        let folder_icon = gtk::Image::new();
        folder_icon.set_icon_name(Some("folder"));
//        folder_icon.set_icon_size(gtk::IconSize::Large);

        hbox.append(&folder_icon);
        hbox.append(&name_label);
        hbox.append(&count_label);
        hbox.append(&separator);

        list_item
            .property_expression("item")
            .chain_property::<CategoryObject>("status")
            .chain_closure::<bool>(gtk::glib::closure!(|_: Option<glib::Object>, status: i64| {
                status == SEPARATOR
            }))
            .bind(&separator, "visible", gtk::Widget::NONE);
        
        list_item
            .property_expression("item")
            .chain_property::<CategoryObject>("status")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, status: i64| {
              match status {
                ALL     => "window-restore-symbolic",
                STOPPED => "media-playback-stop-symbolic",
                VERIFY_QUEUED => "view-refresh-symbolic",
                VERIFYING => "view-refresh-symbolic",
                DOWN_QUEUED => "network-receive-symbolic",
                DOWNLOADING => "arrow-down-symbolic",
                SEED_QUEUED => "network-transmit-symbolic",
                SEEDING => "arrow-up-symbolic",
                FOLDER  => "folder-symbolic",
                ERROR   => "dialog-error-symbolic",
                SEPARATOR => "",
                _ => "dialog-question-symbolic"
              }
            }))
            .bind(&folder_icon, "icon-name", gtk::Widget::NONE);

//        list_item
//            .property_expression("item")
//            .chain_property::<CategoryObject>("is-folder")
//            .bind(&folder_icon, "visible", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<CategoryObject>("status")
            .chain_closure::<bool>(gtk::glib::closure!(|_: Option<glib::Object>, status: i64| {
                status != SEPARATOR
            }))
            .bind(&count_label, "visible", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<CategoryObject>("status")
            .chain_closure::<bool>(gtk::glib::closure!(|_: Option<glib::Object>, status: i64| {
                status != SEPARATOR
            }))
            .bind(&name_label, "visible", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<CategoryObject>("count")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<glib::Object>, count: u64| {
                format!("({})", count)
            }))
            .bind(&count_label, "label", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<CategoryObject>("name")
            .bind(&name_label, "label", gtk::Widget::NONE);
        
//        list_item
//            .property_expression("item")
//            .chain_property::<CategoryObject>("is-folder")
//            .bind(&separator, "visible", gtk::Widget::NONE);
    });

    let category_selection_model = gtk::SingleSelection::new(Some(&category_model));

    category_selection_model.connect_selected_item_notify(clone!(@weak category_filter => move |s| {
      match s.selected_item() {
          Some(item) => {
            let status = item.property_value("status").get::<i64>().unwrap();   
            let is_folder = item.property_value("is-folder").get::<bool>().unwrap();   
            if status == ALL || status == SEPARATOR {
              category_filter.set_filter_func(|_| true);
            } else if is_folder {
              let download_dir = item.property_value("download-dir").get::<String>().unwrap();   
              category_filter.set_filter_func(move |item| {
                  item.property_value("download-dir").get::<String>().unwrap() == download_dir
              });
            } else if status == ERROR {
                category_filter.set_filter_func(move |item| {
                    item.property_value("error").get::<i64>().unwrap() > 0 
                });
            } else {
                category_filter.set_filter_func(move |item| {
                    item.property_value("status").get::<i64>().unwrap() == status 
                });
            }
          },
          None       => {
            category_filter.set_filter_func(|_| true);
          }
      }
    }));
    let category_view = gtk::ListView::new(Some(&category_selection_model), Some(&category_factory));
    category_view.set_vexpand(true);
    category_view.set_margin_top(8);
    category_view.set_css_classes(&["sidebar"]);


//    let controller = gtk::EventControllerKey::new();
 //   torrent_list.add_controller(&controller);
  
    let context_menu_builder = gtk::Builder::from_resource("/org/transgression/torrent_list_context_menu.ui");
    let context_menu_model = context_menu_builder.object::<gio::MenuModel>("menu").expect("can't find context menu");

    // maybe just create a custom popover?
    let context_menu = gtk::PopoverMenu::from_model(Some(&context_menu_model));
    //let context_menu = gtk::PopoverMenu::from_model_full(&context_menu_model, gtk::PopoverMenuFlags::NESTED);
//{
//    let b = gtk::Box::new(gtk::Orientation::Horizontal, 4);
//    let img = gtk::Image::builder().icon_name("system-run-symbolic").build();
//    b.append(&img);
//    let label = gtk::Label::new(Some("hello"));
//    b.append(&label);
//    context_menu.add_child(&b, "hello");
//}

    
    context_menu.set_parent(&torrent_list);
    context_menu.set_position(gtk::PositionType::Bottom);
    context_menu.set_halign(gtk::Align::Start);
    context_menu.set_has_arrow(false);
    context_menu.set_mnemonics_visible(true);

    let action_open_folder = gio::SimpleAction::new("open-folder", None);
    let action_open_term = gio::SimpleAction::new("open-term", None);
    let action_move_torrent = gio::SimpleAction::new("move-torrent", None);
    let action_queue_up = gio::SimpleAction::new("queue-up", None);
    let action_queue_down = gio::SimpleAction::new("queue-down", None);
    let action_queue_top = gio::SimpleAction::new("queue-top", None);
    let action_queue_bottom = gio::SimpleAction::new("queue-bottom", None);
    let action_torrent_start = gio::SimpleAction::new("torrent-start", None);
    let action_torrent_start_now = gio::SimpleAction::new("torrent-start-now", None);
    let action_torrent_stop = gio::SimpleAction::new("torrent-stop", None);
    let action_torrent_verify = gio::SimpleAction::new("torrent-verify", None);
    let action_torrent_reannounce = gio::SimpleAction::new("torrent-reannounce", None);
    let action_torrent_remove = gio::SimpleAction::new("torrent-remove", None);
    let action_torrent_remove_data = gio::SimpleAction::new("torrent-remove-with-data", None);
    window.add_action(&action_open_folder);
    window.add_action(&action_open_term);
    window.add_action(&action_move_torrent);
    window.add_action(&action_queue_up);
    window.add_action(&action_queue_down);
    window.add_action(&action_queue_top);
    window.add_action(&action_queue_bottom);
    window.add_action(&action_torrent_start);
    window.add_action(&action_torrent_stop);
    window.add_action(&action_torrent_verify);
    window.add_action(&action_torrent_remove);
    window.add_action(&action_torrent_remove_data);
    window.add_action(&action_torrent_start_now);
    window.add_action(&action_torrent_reannounce);
    
    let sender = tx2.clone();
    action_torrent_start.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::Start(xs)).expect("can't send torrent_cmd"); 
    }));

    let sender = tx2.clone();
    action_torrent_start_now.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::StartNow(xs)).expect("can't send torrent_cmd"); 
    }));

    let sender = tx2.clone();
    action_torrent_reannounce.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::Reannounce(xs)).expect("can't send torrent_cmd"); 
    }));

    let sender = tx2.clone();
    action_torrent_stop.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::Stop(xs)).expect("can't send torrent_cmd"); 
    }));

    let sender = tx2.clone();
    action_torrent_verify.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::Verify(xs)).expect("can't send torrent_cmd"); 
    }));

    let sender = tx2.clone();
    action_open_folder.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let selection = torrent_selection_model.selection();
      let first = selection.minimum();
      if let Some(item) = torrent_selection_model.item(first) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            sender.blocking_send(TorrentCmd::OpenDlDir(id)).expect("can't send torrent_cmd"); 
      }
    }));

    let sender = tx2.clone();
    action_open_term.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let selection = torrent_selection_model.selection();
      let first = selection.minimum();
      if let Some(item) = torrent_selection_model.item(first) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            sender.blocking_send(TorrentCmd::OpenDlTerm(id)).expect("can't send torrent_cmd"); 
      }
    }));

    let sender = tx2.clone();
    action_queue_up.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::QueueMoveUp(xs)).expect("can't send torrent_cmd"); 
    }));
 
    let sender = tx2.clone();
    action_queue_down.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::QueueMoveDown(xs)).expect("can't send torrent_cmd"); 
    }));
 
    let sender = tx2.clone();
    action_queue_top.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::QueueMoveTop(xs)).expect("can't send torrent_cmd"); 
    }));
 
    let sender = tx2.clone();
    action_queue_bottom.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            xs.push(id);
          }
          i += 1;
      }
      sender.blocking_send(TorrentCmd::QueueMoveBottom(xs)).expect("can't send torrent_cmd"); 
    }));


    let _window = Rc::new(window);
 
    let _category_model = Rc::new(category_model);
    let _category_filter = Rc::new(category_filter);
    let sender = tx2.clone();
    add_button.connect_clicked(clone!(@strong _window, @strong _category_filter => move |_| {
            gtk::glib::MainContext::default().spawn_local(add_torrent_file_dialog(Rc::clone(&_window), sender.clone(), Rc::clone(&_category_filter)));
    }));
    let sender = tx2.clone();
    add_magnet_button.connect_clicked(clone!(@strong _window, @strong _category_filter => move |_| {
            gtk::glib::MainContext::default().spawn_local(add_magnet_dialog(Rc::clone(&_window), sender.clone(), Rc::clone(&_category_filter)));
    }));
    let sender = tx2.clone();
    action_move_torrent.connect_activate(clone!(@strong _window, @strong _category_filter, @strong _category_model, @weak torrent_selection_model => move |_action, _| {
      let selection = torrent_selection_model.selection();
      let first = selection.minimum();
      if let Some(item) = torrent_selection_model.item(first) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            gtk::glib::MainContext::default().spawn_local(move_torrent_dialog(Rc::clone(&_window), sender.clone(), id, Rc::clone(&_category_model), Rc::clone(&_category_filter)));
      }
    }));
   

    let sender = tx2.clone();
    action_torrent_remove.connect_activate(clone!(@strong _window, @weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            xs.push(item);
          }
          i += 1;
      }
      gtk::glib::MainContext::default().spawn_local(deletion_confiramtion_dialog(Rc::clone(&_window), sender.clone(), false, xs));
    }));

   let sender = tx2.clone();
    action_torrent_remove_data.connect_activate(clone!(@strong _window, @weak torrent_selection_model => move |_action, _| {
      let mut xs = vec![];
      let mut i = 0;
      while let Some(item) = torrent_selection_model.item(i) {
          if torrent_selection_model.is_selected(i) {
            xs.push(item);
          }
          i += 1;
      }
      gtk::glib::MainContext::default().spawn_local(deletion_confiramtion_dialog(Rc::clone(&_window), sender.clone(), true, xs));
    }));

    let click = gtk::GestureClick::new();
    click.set_name(Some("torrent-list-click"));
    click.set_button(3);
    click.set_exclusive(true);
    torrent_list.add_controller(&click);

    click.connect_pressed(clone!(@weak context_menu, @weak torrent_list => move |click, n_press, x, y| {
      if let Some(event) = click.current_event()  {
         if n_press == 1 && event.triggers_context_menu() {
           if let Some(sequence) = click.current_sequence() {
             click.set_sequence_state(&sequence, gtk::EventSequenceState::Claimed);
           }

           if let Some(widget) = torrent_list.pick(x, y, gtk::PickFlags::DEFAULT) {
             let mut w = widget;
             while let Some(parent) = w.parent() {
                 if parent.css_name() == "row" {
                   let res = parent.activate_action("listitem.select", Some(&(false,false).to_variant())); 
                   println!("{:?}", res);
                   let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 10, 10);
                   context_menu.set_pointing_to(Some(&rect));
                   context_menu.popup();
                   break;
                 }
               w = parent;
             }
           }
         }
      }
    }));



    // dynamic list of labels 
    left_pane.append(&category_view);

    // ========================================================================

    main_view.set_valign(gtk::Align::Fill);
    main_view.set_start_child(&left_pane);
    main_view.set_end_child(&right_hbox);
    main_view.set_resize_end_child(true);
    main_view.set_resize_start_child(false);
    main_view.set_shrink_start_child(true);

    let details_notebook = build_bottom_notebook(&details_object, &peers_model, &tracker_model, &file_table);

    container.append(&bottom_pane);
    container.append(&status_line);
    bottom_pane.set_start_child(&main_view);
    bottom_pane.set_end_child(&details_notebook);
    bottom_pane.set_resize_start_child(true);
    bottom_pane.set_resize_end_child(false);
    bottom_pane.set_shrink_end_child(true);
    bottom_pane.set_valign(gtk::Align::Fill);
    bottom_pane.set_vexpand(true);

    entry.connect_search_started(clone!(@weak search_button => move |_| {
        search_button.set_active(true);
    }));

    entry.connect_stop_search(clone!(@weak search_button => move |_| {
        search_button.set_active(false);
    }));

    entry.connect_search_changed(clone!(@weak name_filter => move |entry| {
        if entry.text() != "" {
           name_filter.set_search(Some(&entry.text())); 
        } else {
          name_filter.set_search(None);
        }
    }));

    _window.present();
}


fn build_bottom_notebook(details_object: &TorrentDetailsObject, peers_model: &gio::ListStore, 
                         tracker_model: &gio::ListStore, file_table: &gtk::ColumnView) -> gtk::Notebook {
    let details_notebook = gtk::Notebook::builder()
        .build();

    let details_grid = TorrentDetailsGrid::new(&details_object);
    details_notebook.append_page(&details_grid, Some(&gtk::Label::new(Some("General"))));
    
    let trackers_selection_model = gtk::NoSelection::new(Some(tracker_model));
    let trackers_table = gtk::ColumnView::new(Some(&trackers_selection_model));

    let scrolled_window_b = gtk::ScrolledWindow::builder()
        .min_content_width(360)
        .vexpand(true)
        .child(&trackers_table)
        .build();
    details_notebook.append_page(&scrolled_window_b, Some(&gtk::Label::new(Some("Trackers"))));

    let tier_factory = gtk::SignalListItemFactory::new();
    tier_factory.connect_setup(label_setup::<TrackerObject, u64, _>("tier", |x| format!("{}", x)));
    
    let announce_factory = gtk::SignalListItemFactory::new();
    announce_factory.connect_setup(label_setup::<TrackerObject, String, _>("announce", |x| x));
    
    let peers_factory = gtk::SignalListItemFactory::new();
    peers_factory.connect_setup(label_setup::<TrackerObject, u64, _>("last-announce-peer-count", |x| format!("{}", x)));
    
    let seeder_factory = gtk::SignalListItemFactory::new();
    seeder_factory.connect_setup(label_setup::<TrackerObject, i64, _>("seeder-count", |x| format!("{}", x)));
    
    let leecher_factory = gtk::SignalListItemFactory::new();
    leecher_factory.connect_setup(label_setup::<TrackerObject, i64, _>("leecher-count", |x| format!("{}", x)));
    
    let last_announce_time_factory = gtk::SignalListItemFactory::new();
    last_announce_time_factory.connect_setup(label_setup::<TrackerObject, u64, _>("last-announce-time", format_time));
    
    let last_result_factory = gtk::SignalListItemFactory::new();
    last_result_factory.connect_setup(label_setup::<TrackerObject, String, _>("last-announce-result", |x| x));
    
    let scrape_factory = gtk::SignalListItemFactory::new();
    scrape_factory.connect_setup(label_setup::<TrackerObject, String, _>("scrape", |x| x));
    
    let peers_selection_model = gtk::NoSelection::new(Some(peers_model));
    let peers_table = gtk::ColumnView::new(Some(&peers_selection_model));

    trackers_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Tier")
          .expand(true)
          .factory(&tier_factory)
          .build()
        );

    trackers_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Announce URL")
          .expand(true)
          .factory(&announce_factory)
          .build()
        );

    trackers_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Peers")
          .expand(true)
          .factory(&peers_factory)
          .build()
        );

    trackers_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Seeder Count")
          .expand(true)
          .factory(&seeder_factory)
          .build()
        );

    trackers_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Leecher Count")
          .expand(true)
          .factory(&leecher_factory)
          .build()
        );

    trackers_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Last Announce")
          .expand(true)
          .factory(&last_announce_time_factory)
          .build()
        );

    trackers_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Last Result")
          .expand(true)
          .factory(&last_result_factory)
          .build()
        );

    trackers_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Scrape URL")
          .expand(true)
          .factory(&scrape_factory)
          .build()
        );

    let address_factory = gtk::SignalListItemFactory::new();
    address_factory.connect_setup(label_setup::<PeerObject, String, _>("address", |x| x));
    
    let dl_speed_factory = gtk::SignalListItemFactory::new(); // TODO: generalize
    dl_speed_factory.connect_setup(label_setup::<PeerObject, u64, _>("rate-to-client", |i|  utils::format_download_speed(i.try_into().unwrap())));
    

    let ul_speed_factory = gtk::SignalListItemFactory::new(); // TODO: generalize
    ul_speed_factory.connect_setup(label_setup::<PeerObject, u64, _>("rate-to-peer", |i|  utils::format_download_speed(i.try_into().unwrap())));
                

    let progress_factory = gtk::SignalListItemFactory::new();
    progress_factory.connect_setup(move |_, list_item| {
        let progress = gtk::ProgressBar::new();
        list_item.set_child(Some(&progress));

        list_item
            .property_expression("item")
            .chain_property::<PeerObject>("progress")
            .bind(&progress, "fraction", gtk::Widget::NONE);
        
        progress.set_show_text(true);
    });

    
    let flags_factory = gtk::SignalListItemFactory::new();
    flags_factory.connect_setup(label_setup::<PeerObject, String, _>("flag-str", |x| x));

    let client_factory = gtk::SignalListItemFactory::new();
    client_factory.connect_setup(label_setup::<PeerObject, String, _>("client-name", |x| x));

    let address_col = gtk::ColumnViewColumn::new(Some("Address"), Some(&address_factory));
    let dl_speed_col = gtk::ColumnViewColumn::new(Some("Down Speed"), Some(&dl_speed_factory));
    let up_speed_col = gtk::ColumnViewColumn::new(Some("Up Speed"), Some(&ul_speed_factory));
    let progress_col = gtk::ColumnViewColumn::new(Some("Progress"), Some(&progress_factory));
    let flags_col = gtk::ColumnViewColumn::new(Some("Flags"), Some(&flags_factory));
    let client_col = gtk::ColumnViewColumn::new(Some("Client"), Some(&client_factory));
    address_col.set_expand(true);
    dl_speed_col.set_expand(true);
    up_speed_col.set_expand(true);
    progress_col.set_expand(true);
    flags_col.set_expand(true);
    client_col.set_expand(true);

    peers_table.append_column(&address_col);
    peers_table.append_column(&dl_speed_col);
    peers_table.append_column(&up_speed_col);
    peers_table.append_column(&progress_col);
    peers_table.append_column(&flags_col);
    peers_table.append_column(&client_col);
    peers_table.set_show_row_separators(true);
    peers_table.set_show_column_separators(true);

    
    let scrolled_window = gtk::ScrolledWindow::builder()
        .min_content_width(360)
        .vexpand(true)
        .child(&peers_table)
        .build();

    details_notebook.append_page(&scrolled_window, Some(&gtk::Label::new(Some("Peers"))));
    let files_page = build_bottom_files(file_table, true);
    details_notebook.append_page(&files_page, Some(&gtk::Label::new(Some("Files"))));
    //details_notebook.set_current_page(Some(3));
   
  details_notebook
}



//static file_tree_ref : RefCell<Vec<utils::Node>> = RefCell::new(None);




async fn move_torrent_dialog<W: IsA<gtk::Window>>(window: Rc<W>, sender: mpsc::Sender<TorrentCmd>, id: i64, category_model:Rc<gio::ListStore>, filter:Rc<gtk::CustomFilter>) {
    let model = gtk::StringList::new(&vec![]);
    let mut i = 0;
    while let Some(x) = category_model.item(i) {
//        let name = x.property_value("download-dir").get::<String>().expect("skdfj1");
        let folder = x.property_value("download-dir").get::<String>().expect("skdfj1");
        let is_folder = x.property_value("is-folder").get::<bool>().expect("skdfj1");
        if is_folder {
            model.append(folder.as_str());
        } 
        i +=1 ;
    }
  let dialog = gtk::Dialog::builder()
        .transient_for(&*window)
        .modal(true)
        .build();

  dialog.set_css_classes(&["simple-dialog"]);
  dialog.add_button("Cancel", gtk::ResponseType::Cancel);
  dialog.add_button("Move", gtk::ResponseType::Ok);

  // TODO: add custom widget for dropdown, so user can enter custom path, maybe directory picker?
  let destination = gtk::DropDown::builder()
      .model(&model)
      .build();
  let move_checkbox = gtk::CheckButton::builder().active(true).label("Move the data").tooltip_text("if true, move from previous location. otherwise, search 'location' for files").build(); 
  let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
  vbox.append(&destination);
  vbox.append(&move_checkbox);
  dialog.content_area().append(&vbox); // and a label for destination))
  let response = dialog.run_future().await;
  dialog.close();
  if response == gtk::ResponseType::Ok {
     if let Some(o) = destination.selected_item() {
        let is_move = move_checkbox.is_active();
        let folder = o.property_value("string").get::<String>().expect("should be string"); 
        sender.send(TorrentCmd::Move(vec![id], folder, is_move)).await.expect("failure snd move");

        async_std::task::sleep(std::time::Duration::from_millis(3000)).await;
        filter.changed(gtk::FilterChange::MoreStrict); // now only need to schedule this call
     }
  } 
}

async fn deletion_confiramtion_dialog<W: IsA<gtk::Window>>(window: Rc<W>, sender: mpsc::Sender<TorrentCmd>, with_data: bool, items: Vec<glib::Object>) {
     let msg = if items.len() == 1 {
        let foo = if with_data { "with all local data?"} else { "?" }; 
        let name = items[0].property_value("name").get::<String>().expect("name must");
        format!("Do you want to delete '{}' {}", name, foo)
     } else {
        let foo = if with_data { "torrents and all their data?"} else { "torrents?" }; 
        format!("Do you want to delete {} {}", items.len(), foo)
     };
     let dialog = gtk::MessageDialog::builder()
        .transient_for(&*window)
        .modal(true)
        .buttons(gtk::ButtonsType::OkCancel)
        .text(&msg)
//        .use_markup(true)
        .build();
  dialog.set_css_classes(&["simple-dialog"]);

  let response = dialog.run_future().await;
  dialog.close();

  if response == gtk::ResponseType::Ok {
      let ids = items.iter().map(|x| x.property_value("id").get::<i64>().expect("id must")).collect();
      sender.send(TorrentCmd::Delete(ids, with_data)).await.expect("can't snd rm");
  }
}

async fn add_torrent_file_dialog<W: IsA<gtk::Window>>(_window: Rc<W> , sender: mpsc::Sender<TorrentCmd>, filter:Rc<gtk::CustomFilter>) {
    let dialog = gtk::FileChooserDialog::new(
        Some("Select .torrent file"), Some(&*_window), gtk::FileChooserAction::Open, &[("Open", gtk::ResponseType::Ok), ("Cancel", gtk::ResponseType::Cancel)]);
    let torrent_file_filter = gtk::FileFilter::new();
    torrent_file_filter.add_suffix("torrent");
    dialog.add_filter(&torrent_file_filter);
    let response = dialog.run_future().await;
    dialog.close();
    if response == gtk::ResponseType::Ok {
        if let Some(file) = dialog.file() {
            if let Some(path) = file.path() {
                gtk::glib::MainContext::default().spawn_local(add_torrent_dialog(Rc::clone(&_window), sender, None, Some(path), filter));
            }
        }
    }
}

async fn add_magnet_dialog<W: IsA<gtk::Window>>(_window: Rc<W> , sender: mpsc::Sender<TorrentCmd>, filter:Rc<gtk::CustomFilter>) {
  let dialog = gtk::Dialog::builder()
        .transient_for(&*_window)
        .modal(true)
        .build();

  dialog.add_button("Cancel", gtk::ResponseType::Cancel);
  dialog.add_button("Add", gtk::ResponseType::Ok);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let l = gtk::Label::new(Some("Magnet link: "));
    hbox.append(&l);
    let text = gtk::Text::new();
    hbox.append(&text);
    dialog.content_area().append(&hbox); 
    dialog.set_css_classes(&["simple-dialog"]);

    let clipboard = gtk::gdk::Display::default().expect("surely we must have display").primary_clipboard();
    let res =  clipboard.read_text_future().await;
    if let Result::Ok(maybe_text) = res { 
      if let Some(gstring) = maybe_text {
        let s = gstring.to_string();
        if s.starts_with("magnet:") {
          text.set_text(&s);
        }
      }
    } else {
        println!("can't read text clipboard content");
    }
    let response = dialog.run_future().await;
    dialog.close();
    if response == gtk::ResponseType::Ok {
        let s = text.text().to_string();
        if s.starts_with("magnet:") {
                gtk::glib::MainContext::default().spawn_local(add_torrent_dialog(Rc::clone(&_window), sender, Some(s), None, filter));
        }
    }
}
// now, this is not really necessary, as we have NumericSorter and StringSorter.Except, how to
// reverse those fucking sorters?
fn sort_by_property<'a, T>(property_name: &'static str, desc: bool) -> gtk::CustomSorter  
  where  T:for<'b> glib::value::FromValue<'b>  + std::cmp::PartialOrd {
    gtk::CustomSorter::new(move |x,y|{
        if desc == (x.property_value(property_name).get::<T>().expect("ad") > y.property_value(property_name).get::<T>().expect("ad")) {
            gtk::Ordering::Smaller
        } else if desc == (x.property_value(property_name).get::<T>().expect("ad") < y.property_value(property_name).get::<T>().expect("ad")) {
            gtk::Ordering::Larger
        } else {
            gtk::Ordering::Equal
        }
    })
}

// maybe somehow drop Arc, as we are only going to use it from the main thread.
// On the other hand it locks once per setup call, which means not much (i hope)
fn label_setup<T, R, F>(property_name: &'static str, f: F) -> impl Fn(&gtk::SignalListItemFactory, &gtk::ListItem) -> () 
      where T: IsA<glib::Object>, 
            R: for<'b> gtk::glib::value::FromValue<'b>,
            F: Fn(R) -> String + std::marker::Send + std::marker::Sync + 'static {
      let f1 = std::sync::Arc::new(f);
      move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        let g = std::sync::Arc::clone(&f1);
        
                          list_item
            .property_expression("item")
            .chain_property::<T>(property_name)
            .chain_closure::<String>(gtk::glib::closure!(move |_: Option<gtk::glib::Object>, x: R| {
                g(x)
            }))
            .bind(&label, "label", gtk::Widget::NONE);
      }
}
