mod objects;
mod utils;
mod torrent_details_grid;
mod magnet_tools;
mod notification_utils;
mod command_processor;
use crate::objects::{ TorrentInfo, TorrentDetailsObject, Stats, PeerObject, TrackerObject, FileObject, CategoryObject};
use crate::torrent_details_grid::TorrentDetailsGrid;
use gtk::TreeListRow;
use transg::transmission;
use magnet_tools::magnet_to_metainfo;
use base64;
use magnet_url::Magnet;
use urlencoding;
 
use gtk::prelude::*;
use gtk::Application;
use glib::clone;
use gtk::glib;
use gtk::gio;
use utils::format_time;
//use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use gio::ListStore;
use lazy_static::lazy_static;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
//use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::fs;
use async_std::task;

        use transmission::TransmissionClient;

        
        enum TorrentUpdate {
          Full(serde_json::Value),
          Partial(serde_json::Value, serde_json::Value, u64),
          Stats(transmission::SessionStats),
          FreeSpace(transmission::FreeSpace),
        }

        enum TorrentCmd {
            Tick(u64),
            OpenDlDir(i64),
            OpenDlTerm(i64),
            GetDetails(i64),
            QueueMoveUp(Vec<i64>),
            QueueMoveDown(Vec<i64>),
            QueueMoveTop(Vec<i64>),
            QueueMoveBottom(Vec<i64>),
            Delete(Vec<i64>, bool),
            Start(Vec<i64>),
            StartNow(Vec<i64>),
            Stop(Vec<i64>),
            Verify(Vec<i64>),
            Reannounce(Vec<i64>),
            Move(Vec<i64>, String, bool),
            AddTorrent(Option<String>, Option<String>, Option<String>, bool) // download dir, filename, metainfo, start_paused
        }

        

#[derive(Debug)]
struct TorrentGroupStats {
    num_total: u64,
    num_downloading: u64,
    num_queue_down: u64,
    num_queue_up: u64,
    num_seeding: u64,
    num_checking: u64,
    num_stopped: u64,
    num_queue_checking: u64,
    num_error: u64,
    folders: HashMap<String, u64>,
}

fn empty_group_stats() -> TorrentGroupStats {
  TorrentGroupStats {
      num_total: 0,
      num_downloading: 0,
      num_queue_down: 0,
      num_queue_up: 0,
      num_checking: 0,
      num_queue_checking: 0,
      num_stopped: 0,
      num_seeding: 0,
      num_error: 0,
      folders: HashMap::new(),
    }
}


fn process_folder(s: String) -> String {
                let parts : Vec<&str> = s.split("/").collect();
                if parts.len() > 1 {
                  if parts[parts.len() - 2] == "Downloads" {
                    format!("{}", parts[parts.len()-1])
                  } else {
                    format!("{}/{}", parts[parts.len() - 2], parts[parts.len()-1]) 
                  }
                } else {
                    s
                }

}


lazy_static!{
  static ref TORRENT_INFO_FIELDS: Vec<&'static str> = vec!["id", "name", "status", "percentDone", "error", "errorString", 
                "eta", "queuePosition", "isFinished", "isStalled", "metadataPercentComplete",
                "peersConnected", "rateDownload", "rateUpload", "recheckProgress",
                "sizeWhenDone", "downloadDir", "uploadedEver", "uploadRatio", "addedDate"];
}


const STOPPED: i64 = 0;
const VERIFY_QUEUED: i64 = 1;
const VERIFYING: i64 = 2;
const DOWN_QUEUED: i64 = 3;
const DOWNLOADING: i64 = 4;
const SEED_QUEUED: i64 = 5;
const SEEDING: i64 = 6;
const ALL: i64 = -1;
const SEPARATOR: i64 = -2;
const FOLDER: i64 = -3;
const ERROR: i64 = -4;

const NONE_EXPRESSION: Option<&gtk::Expression> = None;
//const NO_SHOW: u64 = u64::MAX;

fn json_value_to_torrent_info(json: &serde_json::Value) -> TorrentInfo {
    let xs = json.as_array().unwrap(); 
    if xs.len() < 20 {
        println!("js array too short");
        std::process::exit(-1);
    }
                  TorrentInfo::new(
                      xs[0].as_i64().unwrap(),
                      xs[1].as_str().unwrap().to_string(),
                      xs[2].as_i64().unwrap(),
                      xs[3].as_f64().unwrap(),
                      xs[4].as_i64().unwrap(),
                      xs[5].as_str().unwrap().to_string(),
                      xs[6].as_i64().unwrap(),
                      xs[7].as_i64().unwrap(),
                      xs[8].as_bool().unwrap(),
                      xs[9].as_bool().unwrap(),
                      xs[10].as_f64().unwrap(),
                      xs[11].as_i64().unwrap(),
                      xs[12].as_i64().unwrap(),
                      xs[13].as_i64().unwrap(),
                      xs[14].as_f64().unwrap(),
                      xs[15].as_i64().unwrap(),
                      xs[16].as_str().unwrap().to_string(),
                      xs[17].as_i64().unwrap(),
                      xs[18].as_f64().unwrap(),
                      xs[19].as_i64().unwrap()
                      )
}

fn update_torrent_stats(model: &ListStore, category_model: &ListStore) {
                 let mut i = 0;
                 let mut group_stats = empty_group_stats();
                 while let Some(x) = model.item(i) {
                   let status = x.property_value("status").get::<i64>().expect("skdfj");
                   let error = x.property_value("error").get::<i64>().expect("skdfj");
                   if error != 0 {
                       group_stats.num_error += 1;
                   }
                   let folder = x.property_value("download-dir").get::<String>().expect("skdfj1");
                   *group_stats.folders.entry(folder).or_insert(1) += 1;
                   group_stats.num_total += 1;
                   match status {
                     STOPPED => group_stats.num_stopped += 1,
                     VERIFY_QUEUED =>  group_stats.num_queue_checking += 1, 
                     VERIFYING => group_stats.num_checking += 1,
                     DOWN_QUEUED => group_stats.num_queue_down += 1,
                     DOWNLOADING => group_stats.num_downloading += 1,
                     SEED_QUEUED => group_stats.num_queue_up += 1,
                     SEEDING     => group_stats.num_seeding += 1,
                     _ => (),
                   }
                   i += 1;
                 }

                 i = 0;
                 while let Some(x) = category_model.item(i) {
                     if x.property_value("is-folder").get::<bool>().expect("sdkfj") == true {
                     let download_dir = x.property_value("download-dir").get::<String>().expect("skdfk");
                     match group_stats.folders.get(&download_dir) {
                         Some(count) => {
                           x.set_property("count", count.to_value());
                           group_stats.folders.remove(&download_dir);
                         },
                         None =>  { category_model.remove(i); continue }
                     }
                     } else {
                     let n = match x.property_value("status").get::<i64>().expect("skdfk") {
                       ALL     => group_stats.num_total,
                       STOPPED => group_stats.num_stopped,
                       VERIFY_QUEUED =>  group_stats.num_queue_checking, 
                       VERIFYING => group_stats.num_checking,
                       DOWN_QUEUED => group_stats.num_queue_down,
                       DOWNLOADING => group_stats.num_downloading,
                       SEED_QUEUED => group_stats.num_queue_up,
                       SEEDING     => group_stats.num_seeding,
                       ERROR       => group_stats.num_error,
                       _ => 0,
                     };
                     x.set_property("count", n.to_value());
                     }
                     i += 1;
                 }

                 for (key, val) in group_stats.folders.iter() {
                     category_model.append(&CategoryObject::new(process_folder(key.to_string()), *val, FOLDER, true, key.to_string()));
                 }

}

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


    // TODO: move to a separate file
    // TODO: add icons!
    //let selected_torrent_mutex : Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None));
    //let selected_torrent_mutex_read = selected_torrent_mutex.clone();
    //        fucking transmission get torrents 

        use glib::{MainContext, PRIORITY_DEFAULT};

        let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT); 
        let (tx1, rx1) = MainContext::channel(PRIORITY_DEFAULT); 

        rx.attach(None, clone!(@weak model,  @weak category_model, @weak stats => @default-return Continue(false), move |update:TorrentUpdate|{
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
                //vec!["id", "status", "percentDone", "error", "errorString", 
                //"eta", "queuePosition", "isFinished", "isStalled", "metadataPercentComplete",
                //"peersConnected", "rateDownload", "rateUpload", "recheckProgress",
                //"sizeWhenDone", "downloadDir", "uploadedEver", "uploadRatio", "addedDate"];
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
                  }
              }
          }

           Continue(true)
        ));

        rx1.attach(None, clone!(@weak details_object, @weak peers_model, @weak tracker_model, @weak file_table => @default-return Continue(false), move |details:transmission::TorrentDetails|{
            let previous_id = details_object.property_value("id").get::<u64>().unwrap();
            details_object.set_property("id", details.id.to_value());
            details_object.set_property("name", details.name.to_value());
            details_object.set_property("eta", details.eta.to_value());
            details_object.set_property("size-when-done", details.size_when_done.to_value());
            details_object.set_property("seeder-count", details.seeder_count.to_value());
            details_object.set_property("leecher-count", details.leecher_count.to_value());
            details_object.set_property("status", details.status.to_value());
            details_object.set_property("download-dir", details.download_dir.to_value());
            details_object.set_property("comment", details.comment.to_value());
            details_object.set_property("hash-string", details.hash_string.to_value());
            details_object.set_property("rate-download", details.rate_download.to_value());
            details_object.set_property("rate-upload", details.rate_upload.to_value());
            details_object.set_property("upload-ratio", details.upload_ratio.to_value());
            details_object.set_property("seed-ratio-limit", details.seed_ratio_limit.to_value());
            details_object.set_property("priority", details.priority.to_value());
            details_object.set_property("done-date", details.done_date.to_value());
            details_object.set_property("percent-complete", details.percent_complete.to_value());
            details_object.set_property("downloaded-ever", details.downloaded_ever.to_value());
            details_object.set_property("uploaded-ever", details.uploaded_ever.to_value());
            details_object.set_property("corrupt-ever", details.corrupt_ever.to_value());
//            details_object.set_property("labels", details.labels.to_value());
            details_object.set_property("piece-count", details.piece_count.to_value());
            details_object.set_property("pieces", details.pieces.to_value());
            details_object.set_property("error", details.error.to_value());
            details_object.set_property("error-string", details.error_string.to_value());

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
            //println!("{:#?}", details);
                          // y.set_property("status", ys[2].as_i64().unwrap().to_value());
           Continue(true)
        }));

        let (tx2, rx2) = mpsc::channel(); 

        let sender = tx2.clone();
        thread::spawn(move || {
            use tokio::runtime::Runtime;
            let rt = Runtime::new().expect("create tokio runtime");
            let mut i:u64 = 0;
            rt.block_on(async {
                loop {
                  tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                  sender.send(TorrentCmd::Tick(i)).expect("can't send tick");
                  i += 1;
                }
            });
        });

        thread::spawn(move || {
          use tokio::runtime::Runtime;
          let rt = Runtime::new().expect("create tokio runtime");
          let client = TransmissionClient::new(&"http://192.168.1.217:9091/transmission/rpc".to_string());


          rt.block_on(async {
              let response = client.get_all_torrents(&TORRENT_INFO_FIELDS).await.expect("oops1");
              let ts = response.get("arguments").unwrap().get("torrents").unwrap().to_owned(); 
              tx.send(TorrentUpdate::Full(ts)).expect("blah");
          });
          
          loop {
            let cmd = rx2.recv().expect("probably ticker thread panicked");

          rt.block_on(async {
            match cmd {
                TorrentCmd::GetDetails(id) => {
                  let details = client.get_torrent_details(vec![id]).await.expect("oops3"); // TODO: what if id is wrong?
                  if details.arguments.torrents.len() > 0 {
                      let res = tx1.send(details.arguments.torrents[0].to_owned());
                      if res.is_err() {
                          println!("{:#?}", res.err().unwrap());
                      }
                  }                                                                                            
                },
                TorrentCmd::Tick(i) => {
                let foo = client.get_recent_torrents(&TORRENT_INFO_FIELDS).await.expect("oops2");
                let torrents = foo.get("arguments").unwrap().get("torrents").unwrap().to_owned(); 
                let removed  = foo.get("arguments").unwrap().get("removed").unwrap().to_owned(); 
//                println!("Received {} torrents", torrents.as_array().unwrap().len());
                //let num_torrents = torrents.as_array().unwrap().len();
                //if num_torrents < 100 { 
                  tx.send(TorrentUpdate::Partial(torrents, removed, i)).expect("blah");
                //}

                if i % 3 == 0 {
                  let stats = client.get_session_stats().await.expect("boo");
                  tx.send(TorrentUpdate::Stats(stats.arguments)).expect("foo");
                }
                if i % 60 == 0 {
                  let free_space = client.get_free_space("/var/lib/transmission/Downloads").await.expect("brkjf");
                  tx.send(TorrentUpdate::FreeSpace(free_space.arguments)).expect("foo");
                }
                },
                TorrentCmd::OpenDlDir(id) => {
                    let details = client.get_torrent_details(vec![id]).await.expect("oops3"); // TODO: what if id is wrong?
                    if details.arguments.torrents.len() > 0 {
                      let location = details.arguments.torrents[0].download_dir.clone();
                      let my_loc   = location.replace("/var/lib/transmission/Downloads", "/run/mount/transmission/Downloads");
                      let me_loc2  = my_loc.clone();
                      let tree = utils::build_tree(&details.arguments.torrents[0].files);
                      let p = my_loc + "/" + &tree[0].path;  
                      if tree.len() == 1 && fs::read_dir(&p).is_ok() {
                        std::process::Command::new("/home/vitalii/.nix-profile/bin/nautilus")
                         .arg(p)
                         .spawn()
                         .expect("failed to spawn");
                      } else {
                        std::process::Command::new("/home/vitalii/.nix-profile/bin/nautilus")
                         .arg(me_loc2)
                         .spawn()
                         .expect("failed to spawn");

                      }
                    }
                },
                TorrentCmd::OpenDlTerm(id) => { // TODO: refactor both into single function
                    let details = client.get_torrent_details(vec![id]).await.expect("oops3"); // TODO: what if id is wrong?
                    if details.arguments.torrents.len() > 0 {
                      let location = details.arguments.torrents[0].download_dir.clone();
                      let my_loc   = location.replace("/var/lib/transmission/Downloads", "/run/mount/transmission/Downloads");
                      let me_loc2  = my_loc.clone();
                      let tree = utils::build_tree(&details.arguments.torrents[0].files);
                      let p = my_loc + "/" + &tree[0].path;  
                      if tree.len() == 1 && fs::read_dir(&p).is_ok() {
                        std::process::Command::new("/home/vitalii/.nix-profile/bin/alacritty")
                         .arg("--working-directory")
                         .arg(&p)
                         .spawn()
                         .expect("failed to spawn");
                      } else {
                        std::process::Command::new("/home/vitalii/.nix-profile/bin/alacritty")
                         .arg("--working-directory")
                         .arg(&me_loc2)
                         .spawn()
                         .expect("failed to spawn");

                      }
                    }
                },
                TorrentCmd::QueueMoveUp(ids) => {
                    client.queue_move_up(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::QueueMoveDown(ids) => {
                    client.queue_move_down(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::QueueMoveTop(ids) => {
                    client.queue_move_top(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::QueueMoveBottom(ids) => {
                    client.queue_move_bottom(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Delete(ids, delete_local_data) => {
                    client.torrent_remove(ids, delete_local_data).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Start(ids) => {
                    client.torrent_start(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::StartNow(ids) => {
                    client.torrent_start_now(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Stop(ids) => {
                    client.torrent_stop(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Verify(ids) => {
                    client.torrent_verify(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Reannounce(ids) => {
                    client.torrent_reannounce(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Move(ids, location, is_move) => {
                    client.torrent_move(ids, &location, is_move).await.expect("ooph4");
                },
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
                       //  TODO: use gtk notifications
                    //    let _ =  notify("Torrent Added!", "").await; // TODO: add name
                    } else {
                     //   let _ = notify("Error!", "").await;
                      println!("{:?}", res);
                    }
                    println!("{:?}", res);
                }
            }
          });

                  
          }       
          });
   
    //      end of transmission code

    //let id_factory = gtk::SignalListItemFactory::new();
    //id_factory.connect_setup(label_setup::<TorrentInfo, i64, _>("id", |x| format!("{}", x)));

    //id_factory.connect_bind(move |_, _list_item| {
    //    let id = _list_item.property::<TorrentInfo>("item").property_value("id").get::<i64>().expect("skdfj");
    //    let mut xs = visible_torrents_mutex.lock().unwrap();//.push(id);
    //    xs.push(id);
    //});


    //id_factory.connect_unbind(move |_, _list_item| {
    //    let id = _list_item.property::<TorrentInfo>("item").property_value("id").get::<i64>().expect("skdfj");
    //    let mut xs = visible_torrents_mutex_rm.lock().unwrap();  
    //    let pos = xs.iter().position(|&x| x == id);
    //    match pos {
    //      Some(idx) => { xs.swap_remove(idx); },
    //      _ => ()
    //    }
    //});

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

    // location, but right now custom actions:what
    //let download_dir_factory = gtk::SignalListItemFactory::new();
    //download_dir_factory.connect_setup(move |_, list_item| {
    //    let container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    //    //let label = gtk::Label::new(None);
    //    list_item.set_child(Some(&container));

    //    let open_folder = gtk::Button::new();
    //    open_folder.set_icon_name("folder-symbolic");
    //    open_folder.set_has_frame(false);

    //    let terminal = gtk::Button::new();
    //    terminal.set_icon_name("utilities-terminal-symbolic");
    //    terminal.set_has_frame(false);
    //    container.append(&open_folder);
    //    container.append(&terminal);

    //    open_folder.connect_clicked(clone!(@strong folder_cmd_sender, @weak list_item => move |_| {
    //      if let Some(data) = list_item.item() {
    //        let torrent_info = data.downcast::<TorrentInfo>().expect("mut be torrent info");
    //        let id = torrent_info.property_value("id").get::<i64>().expect("id must");
    //        folder_cmd_sender.send(TorrentCmd::OpenDlDir(id)).expect("can't send torrent_cmd"); 
    //      };
    //    }));

    //    terminal.connect_clicked(clone!(@strong folder_cmd_sender, @weak list_item => move |_| {
    //      if let Some(data) = list_item.item() {
    //        let torrent_info = data.downcast::<TorrentInfo>().expect("mut be torrent info");
    //        let id = torrent_info.property_value("id").get::<i64>().expect("id must");
    //        folder_cmd_sender.send(TorrentCmd::OpenDlTerm(id)).expect("can't send torrent_cmd"); 
    //      };
    //    }));

    //});

    //download_dir_factory.connect_bind(move |_, list_item| {
    //    let torrent_info = list_item.item().expect("item must be").downcast::<TorrentInfo>().expect("mut be torrent info");
    //    let container = list_item.child().expect("child must be").downcast::<gtk::Box>().expect("must be box");
    //    let open_folder = container.first_child().expect("first child must be").downcast::<gtk::Button>().expect("must be button");
    //    let term = container.last_child().expect("first child must be").downcast::<gtk::Button>().expect("must be button");
    //    
    //    let download_dir = torrent_info.property_value("download-dir").get::<String>().expect("download-dir must");

    //    open_folder.set_tooltip_text(Some(&download_dir));
    //    term.set_tooltip_text(Some(&download_dir));
    //    // now we need a new thread as we want to open top-folder if it exists, not a parent folder
    //});

    //let added_date_factory = gtk::SignalListItemFactory::new();
    //added_date_factory.connect_setup(move |_, list_item| {
    //    let label = gtk::Label::new(None);
    //    list_item.set_child(Some(&label));

    //    list_item
    //        .property_expression("item")
    //        .chain_property::<TorrentInfo>("added-date")
    //        .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, _d: i64| {
    //            ""
    //        }))
    //        .bind(&label, "label", gtk::Widget::NONE);
    //});
    
    
//    let downloading_filter = gtk::FilterListModel::new(Some(&model), Some(&gtk::CustomFilter::new(|x|{ x.property_value("status").get::<i64>().expect("foo") == 4 })));



//    let _filter = gtk::FilterListModel::new(Some(&_sorter), Some(&gtk::CustomFilter::new(|x|{ x.property_value("rate-upload").get::<i64>().expect("foo") > 0 })));
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
          sender.send(TorrentCmd::GetDetails(_model.item(i).unwrap().property_value("id").get::<i64>().unwrap())).expect("can't send details");
        break;
      }
      i += 1;
    } 
});

    let torrent_list = gtk::ColumnView::new(Some(&torrent_selection_model));
   // let some_builder = gtk::Builder::from_resource("foobar");
   // some_builder.object("menu");

 //   torrent_list.set_single_click_activate(true);
 //   torrent_list.set_enable_rubberband(true);
  //  torrent_list.connect_activate(move |_, i|{
   //   println!("{}", i);
    //});
    

//    let c1 = gtk::ColumnViewColumn::new(Some("id"), Some(&id_factory));
//    let c3 = gtk::ColumnViewColumn::new(Some("Status"), Some(&status_factory));
//    let c9 = gtk::ColumnViewColumn::new(Some("Date added"), Some(&added_date_factory));
    let name_col = gtk::ColumnViewColumn::new(Some("Name"), Some(&name_factory));
    let completion_col = gtk::ColumnViewColumn::new(Some("Completion"), Some(&completion_factory));
    let eta_col = gtk::ColumnViewColumn::new(Some("Eta      "), Some(&eta_factory));
    let num_peers_col = gtk::ColumnViewColumn::new(Some("Peers  "), Some(&num_peers_factory));
    let download_speed_col = gtk::ColumnViewColumn::new(Some("Download     "), Some(&download_speed_factory));
    let upload_speed_col = gtk::ColumnViewColumn::new(Some("Upload      "), Some(&upload_speed_factory));
    let size_col = gtk::ColumnViewColumn::new(Some("Size           "), Some(&total_size_factory));
//    let download_dir_col = gtk::ColumnViewColumn::new(None, Some(&download_dir_factory));
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


//    download_dir_col.set_fixed_width(150);

    torrent_list.append_column(&name_col);
    torrent_list.append_column(&completion_col);
    torrent_list.append_column(&eta_col);
    torrent_list.append_column(&download_speed_col);
    torrent_list.append_column(&upload_speed_col);
    torrent_list.append_column(&num_peers_col);
    torrent_list.append_column(&size_col);
    torrent_list.append_column(&ratio_col);
    torrent_list.append_column(&uploaded_ever_col);
//    torrent_list.append_column(&download_dir_col);

    torrent_list.set_reorderable(true);
    torrent_list.set_show_row_separators(false);
    torrent_list.set_show_column_separators(false);

//    torrent_list.set_single_click_activate(false);


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
      sender.send(TorrentCmd::Start(xs)).expect("can't send torrent_cmd"); 
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
      sender.send(TorrentCmd::StartNow(xs)).expect("can't send torrent_cmd"); 
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
      sender.send(TorrentCmd::Reannounce(xs)).expect("can't send torrent_cmd"); 
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
      sender.send(TorrentCmd::Stop(xs)).expect("can't send torrent_cmd"); 
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
      sender.send(TorrentCmd::Verify(xs)).expect("can't send torrent_cmd"); 
    }));

    let sender = tx2.clone();
    action_open_folder.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let selection = torrent_selection_model.selection();
      let first = selection.minimum();
      if let Some(item) = torrent_selection_model.item(first) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            sender.send(TorrentCmd::OpenDlDir(id)).expect("can't send torrent_cmd"); 
      }
    }));

    let sender = tx2.clone();
    action_open_term.connect_activate(clone!(@weak torrent_selection_model => move |_action, _| {
      let selection = torrent_selection_model.selection();
      let first = selection.minimum();
      if let Some(item) = torrent_selection_model.item(first) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            sender.send(TorrentCmd::OpenDlTerm(id)).expect("can't send torrent_cmd"); 
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
      sender.send(TorrentCmd::QueueMoveUp(xs)).expect("can't send torrent_cmd"); 
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
      sender.send(TorrentCmd::QueueMoveDown(xs)).expect("can't send torrent_cmd"); 
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
      sender.send(TorrentCmd::QueueMoveTop(xs)).expect("can't send torrent_cmd"); 
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
      sender.send(TorrentCmd::QueueMoveBottom(xs)).expect("can't send torrent_cmd"); 
    }));


    let _window = Rc::new(window);
 
    let _category_model = Rc::new(category_model);
    let _category_filter = Rc::new(category_filter);
    let sender = tx2.clone();
    add_button.connect_clicked(clone!(@strong _window, @strong _category_model, @strong _category_filter => move |_| {
            gtk::glib::MainContext::default().spawn_local(add_torrent_file_dialog(Rc::clone(&_window), sender.clone(), Rc::clone(&_category_model), Rc::clone(&_category_filter)));
    }));
    let sender = tx2.clone();
    add_magnet_button.connect_clicked(clone!(@strong _window, @strong _category_model, @strong _category_filter => move |_| {
            gtk::glib::MainContext::default().spawn_local(add_magnet_dialog(Rc::clone(&_window), sender.clone(), Rc::clone(&_category_model), Rc::clone(&_category_filter)));
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
   
    //let s = "magnet:?xt=urn:btih:8B7CC1C65FFC1C4DD4D94C523694F24D137AA579&tr=http%3A%2F%2Fbt.t-ru.org%2Fann%3Fmagnet&dn=%D0%A7%D0%B5%D1%80%D0%B5%D0%B7%20%D0%BF%D1%80%D0%B8%D1%86%D0%B5%D0%BB%20%2F%20%D0%A1%D0%B5%D1%80%D0%B8%D0%B8%3A%201-4%20%D0%B8%D0%B7%204%20(%D0%A1%D0%B5%D1%80%D0%B3%D0%B5%D0%B9%20%D0%9A%D0%BE%D1%80%D0%BE%D1%82%D0%B0%D0%B5%D0%B2)%20%5B2022%2C%20%D0%B4%D1%80%D0%B0%D0%BC%D0%B0%2C%20%D0%B2%D0%BE%D0%B5%D0%BD%D0%BD%D1%8B%D0%B9%2C%20%D0%B8%D1%81%D1%82%D0%BE%D1%80%D0%B8%D1%8F%2C%20SATRip-AVC%5D";
    //let s = "magnet:?xt=urn:btih:B7B90236B797D69ADE946413F076010E71BD860A&tr=http%3A%2F%2Fbt3.t-ru.org%2Fann%3Fmagnet&dn=%D0%91%D0%B8%D1%82%D0%B2%D0%B0%20%D0%B2%20%D0%90%D1%80%D0%B4%D0%B5%D0%BD%D0%BD%D0%B0%D1%85%20%2F%20Battle%20of%20the%20Bulge%20(%D0%9A%D0%B5%D0%BD%20%D0%AD%D0%BD%D0%BD%D0%B0%D0%BA%D0%B8%D0%BD%20%2F%20Ken%20Annakin)%20%5B1965%2C%20%D0%A1%D0%A8%D0%90%2C%20%D0%B2%D0%BE%D0%B5%D0%BD%D0%BD%D1%8B%D0%B9%2C%20%D0%B4%D1%80%D0%B0%D0%BC%D0%B0%2C%20BDRip%201080p%5D%20AVO%20(%D0%A0%D1%8F%D0%B1%D0%BE%D0%B2)%20%2B%20Sub%20Eng%20%2B%20Original%20Eng";
    //gtk::glib::MainContext::default().spawn_local(add_torrent_dialog(Rc::clone(&_window), tx2.clone(), Some(s.to_string()), None, Rc::clone(&_category_model), Rc::clone(&_category_filter)));

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
      //sender.send(TorrentCmd::QueueMoveBottom(xs)).expect("can't send torrent_cmd"); 
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
      //sender.send(TorrentCmd::QueueMoveBottom(xs)).expect("can't send torrent_cmd"); 
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

    //click.connect("pressed", true, move |click, n_press, x, y, _data| {
    //  println!("click");
    //  None
    //});
    
//    controller.connect_modifiers
//    controller.connect_key_released(move |a,b,c,d| {
//      println!("click");
//    });
  //  let closure = glib::RustClosure::new(|_| {
  //      println!("hello");
  //      None
  //    //let x = values[0].get::<i32>().unwrap();
  //    //Some((x + 1).to_value())
  //  });
//    category_view.connect_closure("pressed", true, closure);
// yeah! here we can finally listen to fucking events, and activate on fucking single click!    category_view.add_controller()
//
    // todo: use left align, add margin..


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

//static file_tree_ref : RefCell<Vec<utils::Node>> = RefCell::new(None);

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
    
    //let name_factory = gtk::SignalListItemFactory::new();
    //name_factory.connect_setup(move |_, list_item| {
    //    let label = gtk::Label::new(None);
    //    label.set_halign(gtk::Align::Start);
    //     
    //    list_item.set_child(Some(&label));
    //    
    //    

    //    list_item
    //        .property_expression("item")
    //        .chain_property::<TreeListRow>("item")
    //        .chain_property::<FileObject>("name")
    //        .bind(&label, "label", gtk::Widget::NONE);
    //});
    //
    //file_table.append_column(
    //    &gtk::ColumnViewColumn::builder()
    //      .title("Name")
    //      .expand(true)
    //      .factory(&name_factory)
    //      .build()
    //    );


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
        sender.send(TorrentCmd::Move(vec![id], folder, is_move)).expect("failure snd move");

        task::sleep(std::time::Duration::from_millis(3000)).await;
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
      sender.send(TorrentCmd::Delete(ids, with_data)).expect("can't snd rm");
  }
}

async fn add_torrent_file_dialog<W: IsA<gtk::Window>>(_window: Rc<W> , sender: mpsc::Sender<TorrentCmd>, category_model:Rc<gio::ListStore>, filter:Rc<gtk::CustomFilter>) {
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
                gtk::glib::MainContext::default().spawn_local(add_torrent_dialog(Rc::clone(&_window), sender, None, Some(path), category_model, filter));
            }
        }
    }
}

async fn add_magnet_dialog<W: IsA<gtk::Window>>(_window: Rc<W> , sender: mpsc::Sender<TorrentCmd>, category_model:Rc<gio::ListStore>, filter:Rc<gtk::CustomFilter>) {
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
                gtk::glib::MainContext::default().spawn_local(add_torrent_dialog(Rc::clone(&_window), sender, Some(s), None, category_model, filter));
        }
    }
}


async fn add_torrent_dialog<W: IsA<gtk::Window>>(window: Rc<W>, sender: mpsc::Sender<TorrentCmd>, magnet_url: Option<String>, torrent_file:Option<PathBuf>, category_model:Rc<gio::ListStore>, filter:Rc<gtk::CustomFilter>) {
   use lava_torrent::torrent::v1::Torrent;

//  task::sleep(std::time::Duration::from_millis(2000)).await;
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

  let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);

  let file_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  vbox.append(&file_hbox);

  let l = gtk::Label::new(Some("Torrent file"));
  file_hbox.append(&l);

  let _cancel_fetch = Rc::new(Cell::new(false));
  let _magnet_done = Rc::new(Cell::new(false));
  let file_chooser = gtk::Button::new();
  file_hbox.append(&file_chooser);

  let dialog = gtk::Dialog::builder()
        .transient_for(&*window)
        .modal(true)
        .build();

  dialog.add_button("Cancel", gtk::ResponseType::Cancel);
  dialog.add_button("Add", gtk::ResponseType::Ok);

  let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  let l = gtk::Label::new(Some("Destination"));
  hbox.append(&l);
  let destination = gtk::DropDown::builder().model(&model).build();
  hbox.append(&destination);
  vbox.append(&hbox);

  let file_table = gtk::ColumnView::new(None::<&gtk::NoSelection>);
  let scrolled_files = build_bottom_files(&file_table, false);


 // let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&files))));
  let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&vec![]))));
  let files_selection = gtk::NoSelection::new(Some(&stupid_model));
  file_table.set_model(Some(&files_selection));


  if let Some(ref path_buf) = torrent_file {
     let pb2 = path_buf.clone();
     let path = pb2.into_boxed_path();
     let s = path.to_str().unwrap();
     let torrent = Torrent::read_from_file(path.clone()).unwrap();
     let files = torrent.files.unwrap().iter().map(|f| transmission::File {name: f.path.as_path().to_str().unwrap().to_string(), length: f.length as u64, bytes_completed: 0  } ).collect();
     let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&files))));
     let files_selection = gtk::NoSelection::new(Some(&stupid_model));
     file_table.set_model(Some(&files_selection));
     file_chooser.set_label(&s);
  }
  let magnet_data =  Rc::new(RefCell::new(None));
  let progressbar = gtk::ProgressBar::new();
  let _progressbar = Rc::new(progressbar);
  if let Some(ref magnet) = magnet_url {
      let foo : String = magnet.chars().into_iter().take(60).collect();
      let _file_table = Rc::new(file_table);
      if let Result::Ok(m) = Magnet::new(magnet) { 
          let s = m.dn.as_ref().map(|l| urlencoding::decode(l).expect("UTF-8").into_owned());
//        println!("{}", m.dn.expect("foo"));
        file_chooser.set_label(&s.unwrap_or(foo));
        gtk::glib::MainContext::default().spawn_local(pulse_progress(Rc::clone(&_progressbar), Rc::clone(&_magnet_done)));
        gtk::glib::MainContext::default().spawn_local(fetch_magnet_link(magnet.to_string(), Rc::clone(&_file_table), Rc::clone(&_cancel_fetch), Rc::clone(&_magnet_done), Rc::clone(&magnet_data)));
      } else {
         // FIXME: report error on wrong magnets..
      }
  }

  scrolled_files.set_min_content_width(580);
  scrolled_files.set_min_content_height(320);
  vbox.append(&scrolled_files);
  vbox.append(&*_progressbar);

  

  let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
 // let l = gtk::Label::new(Some("Torrent priority"));
 // hbox.append(&l);
//  let priority = gtk::DropDown::builder().build();
//  hbox.append(&priority);
  vbox.append(&hbox);

  let start_paused_checkbox = gtk::CheckButton::builder().active(false).label("Start paused").build(); 
  vbox.append(&start_paused_checkbox);
  let delete_torrent_file = gtk::CheckButton::builder().active(true).label("Delete torrent file").build(); 
  vbox.append(&delete_torrent_file);
  delete_torrent_file.set_sensitive(false);

  dialog.content_area().append(&vbox); // and a label for destination))
  dialog.set_css_classes(&["simple-dialog"]);
  let response = dialog.run_future().await;
  dialog.close();
  _cancel_fetch.set(true);
  _magnet_done.set(true);
  if response == gtk::ResponseType::Ok {
       let start_paused =start_paused_checkbox.is_active();
       let folder = destination.selected_item().map(|x| x.property_value("string").get::<String>().expect("should be string")); 
       if let Some(path_buf) = torrent_file {
         let buf = std::fs::read(path_buf).expect("file invalid");
         let metainfo = base64::encode(buf);
         sender.send(TorrentCmd::AddTorrent(folder, None, Some(metainfo), start_paused)).expect("failure snd move");
       } else if let Some(data) = &*magnet_data.borrow() {
         let metainfo = base64::encode(data);
         sender.send(TorrentCmd::AddTorrent(folder, None, Some(metainfo), start_paused)).expect("failure snd move");
       } else if let Some(url) = magnet_url {
         sender.send(TorrentCmd::AddTorrent(folder, Some(url.to_string()), None, start_paused)).expect("failure snd move");
       } else {
           println!("Error adding torrent. No magnet link and no torrent file is specified");
       }

        task::sleep(std::time::Duration::from_millis(3000)).await;
        filter.changed(gtk::FilterChange::LessStrict); // unnecesserry as we adding the torrent  
    
  } 
}

async fn pulse_progress(_progressbar: Rc<gtk::ProgressBar>, _magnet_done: Rc<Cell<bool>>) {
    loop {
        if _magnet_done.get() {
            break;
        }
        _progressbar.pulse();
       task::sleep(std::time::Duration::from_millis(200)).await;
    }
    _progressbar.set_fraction(1.0);
}

async fn fetch_magnet_link(uri: String, _file_table: Rc<gtk::ColumnView>, cancellation: Rc<Cell<bool>>, done: Rc<Cell<bool>>, magnet_data: Rc<RefCell<Option<Vec<u8>>>>) {
    use lava_torrent::torrent::v1::Torrent;
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
