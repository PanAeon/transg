mod objects;
mod utils;
use crate::objects::{ TorrentInfo, TorrentDetailsObject, Stats, PeerObject, TrackerObject, FileObject, CategoryObject};
use gtk::TreeListRow;
use transg::transmission;
 
use gtk::prelude::*;
use gtk::Application;
use glib::clone;
use gtk::glib;
use gtk::gio;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use gio::ListStore;
use lazy_static::lazy_static;

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

fn format_eta(secs: i64) -> String {
              if secs == -1 {
                  "".to_string()
              } else if secs == -2 {
                  "∞".to_string()
              } else {
                  let days = secs / 86400;
                  let secs = secs - days * 86400;
                  let hours = secs / 3600;
                  let secs = secs - hours * 3600;
                  let minutes = secs / 60;
                  let secs = secs - minutes * 60;
                  
                  if days > 0 {
                    format!("{}d {}h", days, hours)
                  } else if hours > 0  {
                    format!("{}h {}m", hours, minutes) 
                  } else if minutes > 0 {
                    format!("{}m {}s", minutes, secs) 
                  } else {
                    format!("{}s", secs)
                  }
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
fn format_download_speed(i: i64) -> String { 
                if i == 0 {
                  "".to_string()
                } else if i > 1024*1024 {
                  format!("{} Mb/s", i / (1024 * 1024))
                } else {
                  format!("{} Kb/s", i / 1024)
                }
}

fn format_size(i: i64) -> String {
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

const NONE_EXPRESSION: Option<&gtk::Expression> = None;
const NO_SHOW: u64 = u64::MAX;

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
//                   let folder = process_folder(folder);
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
                     category_model.append(&CategoryObject::new(process_folder(key.to_string()), *val, 0, true, key.to_string()));
                 }

}

fn main() {
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
    
    let files_selection = gtk::NoSelection::new(None::<&gio::ListModel>);

    let window = gtk::ApplicationWindow::new(app);
    window.set_default_size(1920, 1080);
    window.set_title(Some("Transgression"));

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(r#"
    window {
      font-size: 12px;
    }
    .sidebar-category-name {
      font-size: 14px;
    }
    .sidebar-category-count {
      font-size: 12px;
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


    let menu_button = gtk::ToggleButton::new();
    menu_button.set_icon_name("format-justify-fill");
    header_bar.pack_end(&menu_button);

    let search_button = gtk::ToggleButton::new();
    search_button.set_icon_name("system-search-symbolic");
    header_bar.pack_end(&search_button);

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

    let queue_down_button = gtk::Button::new();
    queue_down_button.set_icon_name("go-down");
    header_bar.pack_start(&queue_down_button);

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
    let selected_torrent_mutex : Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None));
    let selected_torrent_mutex_read = selected_torrent_mutex.clone();
    //        fucking transmission get torrents 
    {
        use std::thread;
        use transmission::TransmissionClient;

        
        enum TorrentUpdate {
          Full(serde_json::Value),
          Partial(serde_json::Value, serde_json::Value, u64),
          Stats(transmission::SessionStats, transmission::FreeSpace)
        }


        

        use glib::{MainContext, PRIORITY_DEFAULT};

        let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT); 
        let (tx1, rx1) = MainContext::channel(PRIORITY_DEFAULT); 

        rx.attach(None, clone!(@weak model,  @weak category_model, @weak stats => @default-return Continue(false), move |update:TorrentUpdate|{
          match update {
            TorrentUpdate::Full(xs) => {
              let torrents: Vec<TorrentInfo> = xs.as_array().unwrap()
              .iter()
              .skip(1)
              .map(|it| {
                  json_value_to_torrent_info(it)
              }).collect();
              model.splice(0, 0, &torrents);
              update_torrent_stats(&model, &category_model );
                // mutex is not needed as we do this in the ui thread 
                // *group_stats_mutex.lock().unwrap() = group_stats;
              },
           TorrentUpdate::Partial(xs, removed, update_count) => {
//                  println!(">>update received "); 
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
                  
                  for x in xs.as_array().unwrap().iter().skip(1) {
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
                  TorrentUpdate::Stats(s, free_space) => {
                    stats.set_property("upload", s.upload_speed.to_value());
                    stats.set_property("download", s.download_speed.to_value());
                    stats.set_property("free-space", free_space.size_bytes.to_value());
                  } 
              }
          }

           Continue(true)
        ));

        rx1.attach(None, clone!(@weak details_object, @weak peers_model, @weak tracker_model, @weak files_selection => @default-return Continue(false), move |details:transmission::TorrentDetails|{
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
              let stupid_model = create_file_model(details.files);
              files_selection.set_model(Some(&stupid_model));
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

        thread::spawn(move || {
          use tokio::runtime::Runtime;

          let rt = Runtime::new().expect("create tokio runtime");
          rt.block_on(async {

              let response = client.get_all_torrents(&TORRENT_INFO_FIELDS).await.expect("oops1");
              let ts = response.get("arguments").unwrap().get("torrents").unwrap().to_owned(); 
              tx.send(TorrentUpdate::Full(ts)).expect("blah");

              let mut i:u64 = 0;

              loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let opt = selected_torrent_mutex_read.lock().unwrap();//.push(id);
                if let Some(id) = *opt {
                    println!("{}", id);
                  let details = client.get_torrent_details(vec![id]).await.expect("oops3"); // FIXME: what if id is wrong?
                  if details.arguments.torrents.len() > 0 {
                      let res = tx1.send(details.arguments.torrents[0].to_owned());
                      if res.is_err() {
                          println!("{:#?}", res.err().unwrap());
                      }
                  }                                                                                            
                }
                drop(opt);

                let foo = client.get_recent_torrents(&TORRENT_INFO_FIELDS).await.expect("oops2");
                let torrents = foo.get("arguments").unwrap().get("torrents").unwrap().to_owned(); 
                let removed  = foo.get("arguments").unwrap().get("removed").unwrap().to_owned(); 
//                println!("Received {} torrents", torrents.as_array().unwrap().len());
                tx.send(TorrentUpdate::Partial(torrents, removed, i)).expect("blah");
                if i % 15 == 0 {
                  let stats = client.get_session_stats().await.expect("boo");
                  let free_space = client.get_free_space("/var/lib/transmission/Downloads").await.expect("brkjf");
                  tx.send(TorrentUpdate::Stats(stats.arguments, free_space.arguments)).expect("foo");
                }
                i += 1;
              }
          })
        });
   }
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
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        label.set_halign(gtk::Align::Start);

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("name")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, name: &str| {
                format!("^ {}", name)
            }))
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
                format_eta(secs)
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
                format_download_speed(i)
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
                format_download_speed(i)
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
                format_size(i)
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

    let download_dir_factory = gtk::SignalListItemFactory::new();
    download_dir_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<TorrentInfo>("download-dir")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, d: String| {
                process_folder(d)
            }))
            .bind(&label, "label", gtk::Widget::NONE);
    });


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
torrent_selection_model.connect_selection_changed(move |_model, _pos, _num_items| {
    let last = _pos + _num_items;
    let mut i = _pos;
    while i <= last {
      if _model.is_selected(i) {
          *selected_torrent_mutex.lock().unwrap() = Some(_model.item(i).unwrap().property_value("id").get::<i64>().unwrap());
        break;
      }
      i += 1;
    } 
//  println!("{} {} {}", x, y, z);
});
  //  torrent_selection_model.connect_selected_item_notify(move |s| {
  //    match s.selected_item() {
  //        Some(item) => {},
  //        None       => {}
  //    }
  //  });


    let torrent_list = gtk::ColumnView::new(Some(&torrent_selection_model));

//    torrent_list.set_single_click_activate(true);
 //   torrent_list.set_enable_rubberband(true);
  //  torrent_list.connect_activate(move |_, i|{
   //   println!("{}", i);
    //});
    

//    let c1 = gtk::ColumnViewColumn::new(Some("id"), Some(&id_factory));
//    let c3 = gtk::ColumnViewColumn::new(Some("Status"), Some(&status_factory));
//    let c9 = gtk::ColumnViewColumn::new(Some("Date added"), Some(&added_date_factory));
    let name_col = gtk::ColumnViewColumn::new(Some("Name"), Some(&name_factory));
    let completion_col = gtk::ColumnViewColumn::new(Some("Completion"), Some(&completion_factory));
    let eta_col = gtk::ColumnViewColumn::new(Some("Eta  "), Some(&eta_factory));
    let num_peers_col = gtk::ColumnViewColumn::new(Some("Peers  "), Some(&num_peers_factory));
    let download_speed_col = gtk::ColumnViewColumn::new(Some("Download     "), Some(&download_speed_factory));
    let upload_speed_col = gtk::ColumnViewColumn::new(Some("Upload      "), Some(&upload_speed_factory));
    let size_col = gtk::ColumnViewColumn::new(Some("Size           "), Some(&total_size_factory));
    let download_dir_col = gtk::ColumnViewColumn::new(Some("Download dir   "), Some(&download_dir_factory));
    let uploaded_ever_col = gtk::ColumnViewColumn::new(Some("Uploaded     "), Some(&uploaded_ever_factory));
    let ratio_col = gtk::ColumnViewColumn::new(Some("Ratio    "), Some(&ratio_factory));

    name_col.set_resizable(true);
    name_col.set_expand(true);

    completion_col.set_resizable(true);
    eta_col.set_resizable(true);
    download_speed_col.set_resizable(true);
    upload_speed_col.set_resizable(true);
    size_col.set_resizable(true);
    eta_col.set_resizable(true);
    ratio_col.set_resizable(true);
    uploaded_ever_col.set_resizable(true);
//    download_dir_col.set_resizable(true);

    download_dir_col.set_fixed_width(150);

    torrent_list.append_column(&name_col);
    torrent_list.append_column(&completion_col);
    torrent_list.append_column(&eta_col);
    torrent_list.append_column(&download_speed_col);
    torrent_list.append_column(&upload_speed_col);
    torrent_list.append_column(&num_peers_col);
    torrent_list.append_column(&size_col);
    torrent_list.append_column(&ratio_col);
    torrent_list.append_column(&uploaded_ever_col);
    torrent_list.append_column(&download_dir_col);

    torrent_list.set_reorderable(false);
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
   let upload_value_label = gtk::Label::builder().label("").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   stats.property_expression("upload")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                format_download_speed(i.try_into().unwrap())
            }))
       .bind(&upload_value_label, "label", gtk::Widget::NONE);
   let download_line_label = gtk::Label::builder().label(" Download: ").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   let download_value_label = gtk::Label::builder().label("").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   stats.property_expression("download")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                format_download_speed(i.try_into().unwrap())
            }))
       .bind(&download_value_label, "label", gtk::Widget::NONE);
   let free_space_label = gtk::Label::builder().label(" Free space: ").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   let free_space_value_label = gtk::Label::builder().label("").halign(gtk::Align::End).valign(gtk::Align::Center).build();
   stats.property_expression("free-space")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                format_size(i.try_into().unwrap())
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
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
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
            .chain_property::<CategoryObject>("is-folder")
            .bind(&folder_icon, "visible", gtk::Widget::NONE);

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

    let details_notebook = build_bottom_notebook(&details_object, &peers_model, &tracker_model, &files_selection);

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

    window.present();
}

fn build_bottom_notebook(details_object: &TorrentDetailsObject, peers_model: &gio::ListStore, 
                         tracker_model: &gio::ListStore, files_selection: &gtk::NoSelection) -> gtk::Notebook {
    let details_notebook = gtk::Notebook::builder()
        .build();

    let general_page = gtk::Grid::builder().column_spacing(10).row_spacing(10).vexpand(true).build();
    
    general_page.set_row_homogeneous(false);
    general_page.set_column_homogeneous(false);
    general_page.set_margin_start(10);
    general_page.set_margin_end(10);
    general_page.set_margin_top(10);
    general_page.set_margin_bottom(10);


    let name_l = gtk::Label::new(Some("Name"));
    name_l.set_css_classes(&["details-label"]);
    name_l.set_halign(gtk::Align::Start);

    general_page.attach(&name_l, 0, 0, 1, 1);
    let name_label = gtk::Label::new(None);
    name_label.set_halign(gtk::Align::Start);
    general_page.attach(&name_label, 1, 0, 5, 1);
    
    details_object.property_expression("name")
       .bind(&name_label, "label", gtk::Widget::NONE);

    let size_l = gtk::Label::new(Some("Size"));
    size_l.set_css_classes(&["details-label"]);
    size_l.set_halign(gtk::Align::Start);
    general_page.attach(&size_l, 0, 1, 1, 1);
    let size_label = gtk::Label::new(None);
    size_label.set_halign(gtk::Align::Start);
    general_page.attach(&size_label, 1, 1, 1, 1);
    
    details_object.property_expression("size-when-done")
       .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
            format_size(i.try_into().unwrap())
        }))
       .bind(&size_label, "label", gtk::Widget::NONE);

    let eta_l = gtk::Label::new(Some("ETA"));
    eta_l.set_css_classes(&["details-label"]);
    eta_l.set_halign(gtk::Align::Start);
    general_page.attach(&eta_l, 0, 2, 1, 1);
    let eta_label = gtk::Label::new(None);
    eta_label.set_halign(gtk::Align::Start);
    general_page.attach(&eta_label, 1, 2, 1, 1);
    
    details_object.property_expression("eta")
       .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: i64| {
           format_eta(i)
        }))
       .bind(&eta_label, "label", gtk::Widget::NONE);

    let seeders_l = gtk::Label::new(Some("Seeders"));
    seeders_l.set_css_classes(&["details-label"]);
    seeders_l.set_halign(gtk::Align::Start);
    general_page.attach(&seeders_l, 0, 3, 1, 1);
    let seeders_label = gtk::Label::new(None);
    seeders_label.set_halign(gtk::Align::Start);
    general_page.attach(&seeders_label, 1, 3, 1, 1);
    
    details_object.property_expression("seeder-count")
       .bind(&seeders_label, "label", gtk::Widget::NONE);

    let leechers_l = gtk::Label::new(Some("Leechers"));
    leechers_l.set_css_classes(&["details-label"]);
    leechers_l.set_halign(gtk::Align::Start);
    general_page.attach(&leechers_l, 0, 4, 1, 1);
    let leechers_label = gtk::Label::new(None);
    leechers_label.set_halign(gtk::Align::Start);
    general_page.attach(&leechers_label, 1, 4, 1, 1);
    
    details_object.property_expression("leecher-count")
       .bind(&leechers_label, "label", gtk::Widget::NONE);

    let status_l = gtk::Label::new(Some("Status"));
    status_l.set_css_classes(&["details-label"]);
    status_l.set_halign(gtk::Align::Start);
    general_page.attach(&status_l, 0, 5, 1, 1);
    let status_label = gtk::Label::new(None);
    status_label.set_halign(gtk::Align::Start);
    general_page.attach(&status_label, 1, 5, 1, 1);
    
    details_object.property_expression("status")
       .bind(&status_label, "label", gtk::Widget::NONE);

    let location = gtk::Label::new(Some("Location"));
    location.set_css_classes(&["details-label"]);
    location.set_halign(gtk::Align::Start);
    general_page.attach(&location, 0, 6, 1, 1);
    let location_label = gtk::Label::new(None);
    location_label.set_halign(gtk::Align::Start);
    general_page.attach(&location_label, 1, 6, 4, 1);
    
    details_object.property_expression("download-dir")
       .bind(&location_label, "label", gtk::Widget::NONE);

    let comment_l = gtk::Label::new(Some("Comment"));
    comment_l.set_css_classes(&["details-label"]);
    comment_l.set_halign(gtk::Align::Start);
    general_page.attach(&comment_l, 0, 7, 1, 1);
    let comment_label = gtk::Label::new(None);
    comment_label.set_halign(gtk::Align::Start);
    general_page.attach(&comment_label, 1, 7, 4, 1);
    
    details_object.property_expression("comment")
       .bind(&comment_label, "label", gtk::Widget::NONE);

    let hash_l = gtk::Label::new(Some("Hash"));
    hash_l.set_css_classes(&["details-label"]);
    hash_l.set_halign(gtk::Align::Start);
    general_page.attach(&hash_l, 0, 8, 1, 1);
    let hash_label = gtk::Label::new(None);
    hash_label.set_halign(gtk::Align::Start);
    general_page.attach(&hash_label, 1, 8, 4, 1);
    
    details_object.property_expression("hash-string")
       .bind(&hash_label, "label", gtk::Widget::NONE);

    let rate_down_label = gtk::Label::new(Some("Rate Down"));
    rate_down_label.set_css_classes(&["details-label"]);
    rate_down_label.set_halign(gtk::Align::Start);
    general_page.attach(&rate_down_label, 2, 1, 1, 1);
    let rate_down = gtk::Label::new(None);
    rate_down.set_halign(gtk::Align::Start);
    general_page.attach(&rate_down, 3, 1, 1, 1);
    
    details_object.property_expression("rate-download")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                format_download_speed(i.try_into().unwrap())
            }))
       .bind(&rate_down, "label", gtk::Widget::NONE);

    let rate_up_l = gtk::Label::new(Some("Rate Up"));
    rate_up_l.set_css_classes(&["details-label"]);
    rate_up_l.set_halign(gtk::Align::Start);
    general_page.attach(&rate_up_l, 2, 2, 1, 1);
    let rate_up_label = gtk::Label::new(None);
    rate_up_label.set_halign(gtk::Align::Start);
    general_page.attach(&rate_up_label, 3, 2, 1, 1);
    
    details_object.property_expression("rate-upload")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                format_download_speed(i.try_into().unwrap())
            }))
       .bind(&rate_up_label, "label", gtk::Widget::NONE);

    let ratio_label = gtk::Label::new(Some("Ratio"));
    ratio_label.set_css_classes(&["details-label"]);
    ratio_label.set_halign(gtk::Align::Start);
    general_page.attach(&ratio_label, 2, 3, 1, 1);
    let ratio_l = gtk::Label::new(None);
    ratio_l.set_halign(gtk::Align::Start);
    general_page.attach(&ratio_l, 3, 3, 1, 1);
    
    details_object.property_expression("upload-ratio")
       .bind(&ratio_l, "label", gtk::Widget::NONE);

    let ratio_limit_label = gtk::Label::new(Some("Ratio limit"));
    ratio_limit_label.set_css_classes(&["details-label"]);
    ratio_limit_label.set_halign(gtk::Align::Start);
    general_page.attach(&ratio_limit_label, 2, 4, 1, 1);
    let ratio_limit_l = gtk::Label::new(None);
    ratio_limit_l.set_halign(gtk::Align::Start);
    general_page.attach(&ratio_limit_l, 3, 4, 1, 1);
    
    details_object.property_expression("seed-ratio-limit")
       .bind(&ratio_limit_l, "label", gtk::Widget::NONE);

    let priority_label = gtk::Label::new(Some("Priority"));
    priority_label.set_css_classes(&["details-label"]);
    priority_label.set_halign(gtk::Align::Start);
    general_page.attach(&priority_label, 2, 5, 1, 1);
    let priority_l = gtk::Label::new(None);
    priority_l.set_halign(gtk::Align::Start);
    general_page.attach(&priority_l, 3, 5, 1, 1);
    
    details_object.property_expression("priority")
       .bind(&priority_l, "label", gtk::Widget::NONE);

    let completed_label = gtk::Label::new(Some("Completed"));
    completed_label.set_css_classes(&["details-label"]);
    completed_label.set_halign(gtk::Align::Start);
    general_page.attach(&completed_label, 4, 1, 1, 1);
    let completed_l = gtk::Label::new(None);
    completed_l.set_halign(gtk::Align::Start);
    general_page.attach(&completed_l, 5, 1, 1, 1);
    
    details_object.property_expression("percent-complete")
       .bind(&completed_l, "label", gtk::Widget::NONE);

    let downloaded_label = gtk::Label::new(Some("Downloaded"));
    downloaded_label.set_css_classes(&["details-label"]);
    downloaded_label.set_halign(gtk::Align::Start);
    general_page.attach(&downloaded_label, 4, 2, 1, 1);
    let downloaded_l  = gtk::Label::new(None);
    downloaded_l .set_halign(gtk::Align::Start);
    general_page.attach(&downloaded_l , 5, 2, 1, 1);
    
    details_object.property_expression("downloaded-ever")
       .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
            format_size(i.try_into().unwrap())
        }))
       .bind(&downloaded_l, "label", gtk::Widget::NONE);

    let uploaded_label = gtk::Label::new(Some("Uploaded"));
    uploaded_label.set_css_classes(&["details-label"]);
    uploaded_label.set_halign(gtk::Align::Start);
    general_page.attach(&uploaded_label, 4, 3, 1, 1);
    let uploaded_l = gtk::Label::new(None);
    uploaded_l.set_halign(gtk::Align::Start);
    general_page.attach(&uploaded_l, 5, 3, 1, 1);
    
    details_object.property_expression("uploaded-ever")
       .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
            format_size(i.try_into().unwrap())
        }))
       .bind(&uploaded_l, "label", gtk::Widget::NONE);

    let corrupted_label = gtk::Label::new(Some("Corrupted"));
    corrupted_label.set_css_classes(&["details-label"]);
    corrupted_label.set_halign(gtk::Align::Start);
    general_page.attach(&corrupted_label, 4, 4, 1, 1);
    let corrupted_l = gtk::Label::new(None);
    corrupted_l.set_halign(gtk::Align::Start);
    general_page.attach(&corrupted_l, 5, 4, 1, 1);
    
    details_object.property_expression("corrupt-ever")
       .bind(&corrupted_l, "label", gtk::Widget::NONE);

    let completed_at_label = gtk::Label::new(Some("Completed At"));
    completed_at_label.set_css_classes(&["details-label"]);
    completed_at_label.set_halign(gtk::Align::Start);
    general_page.attach(&completed_at_label, 4, 5, 1, 1);

    let completed_at_l = gtk::Label::new(None);
    completed_at_l.set_halign(gtk::Align::Start);
    general_page.attach(&completed_at_l, 5, 5, 1, 1);
    
    details_object.property_expression("done-date")
       .bind(&completed_at_l, "label", gtk::Widget::NONE);


    details_notebook.append_page(&general_page, Some(&gtk::Label::new(Some("General"))));
    
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
                format_download_speed(i.try_into().unwrap())
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
                format_download_speed(i.try_into().unwrap())
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
    let files_page = build_bottom_files(files_selection);
    details_notebook.append_page(&files_page, Some(&gtk::Label::new(Some("Files"))));
    details_notebook.set_current_page(Some(3));
   
  details_notebook
}

fn calculate_total_size(length: &mut u64, completed: &mut u64, node: &utils::Node, files: &HashMap<String, &transmission::File>) {
  if let Some(f) = files.get(&node.path) {
    *length += f.length;
    *completed += f.bytes_completed;
  } else {
    for n in &node.children {
        calculate_total_size(length, completed, &n, files);
    }
  }
}

// TODO: right_now may be not optimal, that's fine
fn build_file_objects_rec(tree: &Vec<utils::Node>, files: &HashMap<String, &transmission::File>, res: &mut HashMap<String, FileObject>) {
  for node in tree {
      if let Some(f) = files.get(&node.path) {
        // terminal
        let fraction = f.bytes_completed as f64 / f.length as f64;
        res.insert(node.path.clone(), FileObject::new(&node.data, &f.length, &fraction, true, 3) );
      } else {
          let mut length: u64 = 0;
          let mut completed: u64 = 0;
          calculate_total_size(&mut length, &mut completed, &node, &files);
          let fraction = completed as f64 / length as f64;
          res.insert(node.path.clone(), FileObject::new(&node.path, &length, &fraction, true, 3) ); // FIXME: here, add also name
          build_file_objects_rec(&node.children, files, res);
      }
  }
} 

fn get_children(tree: Vec<utils::Node>, mut path: Vec<&str>) -> Vec<utils::Node> {
  for n in tree {
    if n.data == path[0] {
        if path.len() == 1 {
            return n.children;
        } else {
            path.remove(0);
            return get_children(n.children, path);
        }
    }
  }
  vec![]
}

fn create_file_model(files: Vec<transmission::File>) ->  gtk::TreeListModel {
//    let files = [transmission::File { name: "Populous The Beginning [GOG]/setup_populous_the_beginning_1.0.0.33.exe".to_string(), length: 329689647, bytes_completed: 329689647 }, transmission::File { name: "Populous The Beginning [GOG]/Bonus Content.rar".to_string(), length: 2576303, bytes_completed: 2576303 }, transmission::File { name: "Populous The Beginning [GOG]/Hardware Mode Fix.zip".to_string(), length: 1514303, bytes_completed: 1514303 }];
    let xs : Vec<Vec<&str>> = files.iter().map(|f| f.name.split("/").collect()).collect();
    let tree = utils::build_tree("", xs);
    let mut files_by_name = HashMap::new();
    for f in &files {
        files_by_name.insert(f.name.clone(), f);
    }
    let mut file_objects_by_name = HashMap::new();
    build_file_objects_rec(&tree, &files_by_name, &mut file_objects_by_name);

    let m0 = gio::ListStore::new(FileObject::static_type());
    let mut v0 : Vec<FileObject> = vec![];
    for node in &tree { 
        v0.push(file_objects_by_name.get(&node.path).unwrap().to_owned());
    }
    m0.splice(0, 0, &v0);

    let fun =  move |x:&glib::Object|{
      let path = x.property_value("path").get::<String>().unwrap();
      let p : Vec<&str> = path.split("/").collect();
      let xs : Vec<Vec<&str>> = files.iter().map(|f| f.name.split("/").collect()).collect();
      let tree = utils::build_tree("", xs);
      let children = get_children(tree, p);
      if children.len() > 0 {

    let m0 = gio::ListStore::new(FileObject::static_type());
    
    let mut v0 : Vec<FileObject> = vec![];
    for node in &children { 
//        println!("{:?}", file_objects_by_name);
        v0.push(file_objects_by_name.get(&node.path).unwrap().to_owned());
    }
    m0.splice(0, 0, &v0);
//    let model = gtk::TreeListModel::new(&m0, false, true,fun);
        Some(m0.upcast())
      } else {
          None
      }
    };
  

    let model = gtk::TreeListModel::new(&m0, false, true,fun);
    model
}



fn build_bottom_files(selection_model: &gtk::NoSelection) -> gtk::ScrolledWindow {
  // foo.set_data();

  let file_table = gtk::ColumnView::new(Some(selection_model));

    let name_factory = gtk::SignalListItemFactory::new();
    name_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
         
        let expander = gtk::TreeExpander::new();
        list_item.set_child(Some(&expander));
        
        expander.set_child(Some(&label));
        
        list_item
            .property_expression("item")
            .bind(&expander, "list-row", gtk::Widget::NONE);

        expander
            .property_expression("item")
            .chain_property::<FileObject>("path")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    
    file_table.append_column(
        &gtk::ColumnViewColumn::builder()
          .title("Name")
          .expand(true)
          .factory(&name_factory)
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
                format_size(i.try_into().unwrap())
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
        .child(&file_table)
        .build()
}




