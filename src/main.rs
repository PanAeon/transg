mod objects;
mod utils;
mod torrent_details_grid;
use crate::objects::{ TorrentInfo, TorrentDetailsObject, Stats, PeerObject, TrackerObject, FileObject, CategoryObject};
use crate::torrent_details_grid::TorrentDetailsGrid;
use gtk::TreeListRow;
use transg::transmission;
 
use gtk::prelude::*;
use gtk::Application;
use glib::clone;
use gtk::glib;
use gtk::gio;
//use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use gio::ListStore;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::rc::Rc;
//use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::fs;


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
        CategoryObject::new("-".to_string(), 0, SEPARATOR, false, "".to_string()),
    ]);

    let details_object = TorrentDetailsObject::new(
        &u64::MAX, &"".to_string(), &0, &0, &0, &0, &0, &"".to_string(), &"".to_string(), 
        &"".to_string(), &0, &0, &0.0, &0, &0, &0, &0.0, 
        &0, &0, &0, &0, &"".to_string()
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


    let menu_button = gtk::MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    header_bar.pack_end(&menu_button);


    let menu_builder = gtk::Builder::from_resource("/org/transgression/main_menu.ui");
    let menu_model = menu_builder.object::<gio::MenuModel>("menu").expect("can't find menu");
    menu_button.set_menu_model(Some(&menu_model));
    menu_button.set_primary(true);

    let search_button = gtk::ToggleButton::new();
    search_button.set_icon_name("system-search-symbolic");
    header_bar.pack_end(&search_button);

    let sort_button = gtk::ToggleButton::new();
    sort_button.set_icon_name("view-list-symbolic");
    header_bar.pack_end(&sort_button);

    let add_button = gtk::Button::new();
    add_button.set_icon_name("list-add");
    header_bar.pack_start(&add_button);

    let start_button = gtk::Button::new();
    start_button.set_icon_name("media-playback-start");
    header_bar.pack_start(&start_button);


    let pause_button = gtk::Button::new();
    pause_button.set_icon_name("media-playback-pause");
    header_bar.pack_start(&pause_button);

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

    let remove_with_files_button = gtk::Button::new();
    remove_with_files_button.set_icon_name("edit-delete");
    header_bar.pack_start(&remove_with_files_button);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.set_child(Some(&container));


    // TODO: move to a separate file
    // TODO: add icons!
    //let selected_torrent_mutex : Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None));
    //let selected_torrent_mutex_read = selected_torrent_mutex.clone();
    //        fucking transmission get torrents 
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
            QueueMoveBottom(Vec<i64>)
        }


        

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
              torrents.sort_by(|a, b| b.property_value("id").get::<i64>().expect("fkjf").cmp(&a.property_value("id").get::<i64>().expect("xx")));
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
                  xxs.sort_by(|a, b| b.as_array().unwrap()[0].as_i64().unwrap().cmp(&a.as_array().unwrap()[0].as_i64().unwrap()));

                  for x in xxs.iter() {
                      let mut i = 0;
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

        let client = TransmissionClient::new(&"http://192.168.1.217:9091/transmission/rpc".to_string());
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
                //println!("Received {} torrents", torrents.as_array().unwrap().len());
                let num_torrents = torrents.as_array().unwrap().len();
                if num_torrents < 100 { // FIXME: more efficient merge.., maybe by id
                  tx.send(TorrentUpdate::Partial(torrents, removed, i)).expect("blah");
                }

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
                    println!("IDs: {:?}", ids);
                    client.queue_move_bottom(ids).await.expect("oops3"); // TODO: proper error handling 
                    println!("???")
                },
            }
          });

                  
          }       
          });
   
    //      end of transmission code

    let id_factory = gtk::SignalListItemFactory::new();

    id_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("id")
            .bind(&label, "label", gtk::Widget::NONE);
    });

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
        hbox.append(&icon);
        hbox.append(&label);

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("status")
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

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("status")
            .bind(&label, "label", gtk::Widget::NONE);
    });

    let eta_factory = gtk::SignalListItemFactory::new();
    eta_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("eta")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, secs: i64| {
                utils::format_eta(secs)
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });

    let ratio_factory = gtk::SignalListItemFactory::new();
    ratio_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("upload-ratio")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: f64| {
              format!("{:.2}", i)
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });

    let num_peers_factory = gtk::SignalListItemFactory::new();
    num_peers_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("peers-connected")
            .bind(&label, "label", gtk::Widget::NONE);
    });

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
    download_speed_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("rate-download")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: i64| {
                utils::format_download_speed(i)
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });


    let upload_speed_factory = gtk::SignalListItemFactory::new();
    upload_speed_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("rate-upload")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: i64| {
                utils::format_download_speed(i)
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });


    let total_size_factory = gtk::SignalListItemFactory::new();
    total_size_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("size-when-done")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: i64| {
                utils::format_size(i)
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });

    let uploaded_ever_factory = gtk::SignalListItemFactory::new();
    uploaded_ever_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("uploaded-ever")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: i64| {
                if i == 0 {
                  "".to_string()
                } else if i > 1024*1024*1024*1024 {
                  format!("{:.2} Tib", i as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
                } else if i > 1024*1024*1024 {
                  format!("{:.2} Gib", i as f64 / (1024.0 * 1024.0 * 1024.0))
                } else if i > 1024*1024 {
                  format!("{:.2} Mib", i as f64 / (1024.0 * 1024.0))
                } else {
                  format!("{:.2} Kib", i as f64 / 1024.0)
                }
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });

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

    let added_date_factory = gtk::SignalListItemFactory::new();
    added_date_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("added-date")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, _d: i64| {
                ""
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    
//    let downloading_filter = gtk::FilterListModel::new(Some(&model), Some(&gtk::CustomFilter::new(|x|{ x.property_value("status").get::<i64>().expect("foo") == 4 })));


    let _sorter = gtk::SortListModel::new(Some(&model), Some(&gtk::CustomSorter::new(|x,y|{
        if x.property_value("added-date").get::<i64>().expect("ad") > y.property_value("added-date").get::<i64>().expect("ad") {
            gtk::Ordering::Smaller
        } else if x.property_value("added-date").get::<i64>().expect("ad") < y.property_value("added-date").get::<i64>().expect("ad") {
            gtk::Ordering::Larger
        } else {
            gtk::Ordering::Equal
        }
    })));

    let _sorter2 = gtk::SortListModel::new(Some(&model), Some(&gtk::CustomSorter::new(|x,y|{
        if x.property_value("peers-connected").get::<i64>().expect("ad") > y.property_value("peers-connected").get::<i64>().expect("ad") {
            gtk::Ordering::Smaller
        } else if x.property_value("peers-connected").get::<i64>().expect("ad") < y.property_value("peers-connected").get::<i64>().expect("ad") {
            gtk::Ordering::Larger
        } else {
            gtk::Ordering::Equal
        }
    })));
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
    window.add_action(&action_open_folder);
    window.add_action(&action_open_term);
    window.add_action(&action_move_torrent);
    window.add_action(&action_queue_up);
    window.add_action(&action_queue_down);
    window.add_action(&action_queue_top);
    window.add_action(&action_queue_bottom);
    
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
    action_move_torrent.connect_activate(clone!(@strong _window, @weak torrent_selection_model => move |_action, _| {
      let selection = torrent_selection_model.selection();
      let first = selection.minimum();
      if let Some(item) = torrent_selection_model.item(first) {
            let id = item.property_value("id").get::<i64>().expect("id must");
            gtk::glib::MainContext::default().spawn_local(dialog(Rc::clone(&_window)));
      }
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
    tier_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TrackerObject>("tier")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let announce_factory = gtk::SignalListItemFactory::new();
    announce_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TrackerObject>("announce")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let peers_factory = gtk::SignalListItemFactory::new();
    peers_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TrackerObject>("last-announce-peer-count")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let seeder_factory = gtk::SignalListItemFactory::new();
    seeder_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TrackerObject>("seeder-count")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let leecher_factory = gtk::SignalListItemFactory::new();
    leecher_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TrackerObject>("leecher-count")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let last_announe_time_factory = gtk::SignalListItemFactory::new();
    last_announe_time_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TrackerObject>("last-announce-time")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let last_result_factory = gtk::SignalListItemFactory::new();
    last_result_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TrackerObject>("last-announce-result")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let scrape_factory = gtk::SignalListItemFactory::new();
    scrape_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TrackerObject>("scrape")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
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
          .factory(&last_announe_time_factory)
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
    address_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<PeerObject>("address")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    let dl_speed_factory = gtk::SignalListItemFactory::new(); // TODO: generalize
    dl_speed_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<PeerObject>("rate-to-client")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                utils::format_download_speed(i.try_into().unwrap())
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });

    let ul_speed_factory = gtk::SignalListItemFactory::new(); // TODO: generalize
    ul_speed_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<PeerObject>("rate-to-peer")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                utils::format_download_speed(i.try_into().unwrap())
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });

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
    flags_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<PeerObject>("flag-str")
            .bind(&label, "label", gtk::Widget::NONE);
    });

    let client_factory = gtk::SignalListItemFactory::new();
    client_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<PeerObject>("client-name")
            .bind(&label, "label", gtk::Widget::NONE);
    });
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
    let files_page = build_bottom_files(file_table);
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



fn build_bottom_files(file_table: &gtk::ColumnView) -> gtk::ScrolledWindow {

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


async fn dialog<W: IsA<gtk::Window>>(window: Rc<W>) {
  let dialog = gtk::Dialog::builder()
        .transient_for(&*window)
        .modal(true)
        .build();

  dialog.add_button("Cancel", gtk::ResponseType::Cancel);
  dialog.add_button("Move", gtk::ResponseType::Ok);

  // TODO: add custom widget for dropdown, so user can enter custom path, maybe directory picker?
  let destination = gtk::DropDown::builder()
      .build();
  
  dialog.content_area().append(&destination); // and a label for destination))
  let response = dialog.run_future().await;
  dialog.close();
}

