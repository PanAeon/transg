mod torrent_list_model;
mod status_model;
mod transmission_client;
mod folder_model;
mod stats_model;
mod torrent_details_model;
mod file_model;
mod file_stats_model;
mod peer_model;
mod tracker_model; // FIXME: make just one module..
mod tracker_stats_model;

use gtk::prelude::*;
use crate::transmission_client::TorrentDetails;
use gtk::Application;
use glib::clone;
use gtk::glib;
use gtk::gio;
use torrent_list_model::TorrentInfo;
use status_model::StatusInfo;
use folder_model::FolderInfo;
use stats_model::Stats;
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

fn update_torrent_stats(model: &ListStore, status_model: &ListStore, folder_model: &ListStore) {
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
                 while let Some(x) = status_model.item(i) {
                     let n = match x.property_value("id").get::<i64>().expect("skdfk") {
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
                     i += 1;
                 }

                 i = 0;
                 while let Some(x) = folder_model.item(i) {
                     let name = x.property_value("name").get::<String>().expect("skdfk");
                     match group_stats.folders.get(&name) {
                         Some(count) => {
                           x.set_property("count", count.to_value());
                           group_stats.folders.remove(&name);
                           i += 1;
                         },
                         None => if name != "any" { 
                             folder_model.remove(i);
                         } else {
                             i+= 1;
                         }
                     }
                 }

                 for (key, val) in group_stats.folders.iter() {
                     folder_model.append(&FolderInfo::new(key.to_string(), *val));
                 }
                 

}

fn main() {
    let app = gtk::Application::new(
        Some("org.example.HelloWorld"),
        Default::default());

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {

    let  model = gio::ListStore::new(TorrentInfo::static_type());
    let status_model = gio::ListStore::new(StatusInfo::static_type());
    let stats = Stats::new(0, 0, 0);
    status_model.splice(0, 0, &vec![
        StatusInfo::new(ALL,0),
        StatusInfo::new(0,0),
        StatusInfo::new(1,0),
        StatusInfo::new(2,0),
        StatusInfo::new(3,0),
        StatusInfo::new(4,0),
        StatusInfo::new(5,0),
        StatusInfo::new(6,0),
    ]);
    let folder_model = gio::ListStore::new(FolderInfo::static_type());
    folder_model.append(&FolderInfo::new("any".to_string(), NO_SHOW));

    let details_object = torrent_details_model::TorrentDetailsObject::new(
        &0, &"".to_string(), &0, &0, &0, &0, &0, &"".to_string(), &"".to_string(), 
        &"".to_string(), &0, &0, &0.0, &0, &0, &0, &0.0, 
        &0, &0, &0, &0, &"".to_string()
        );

    let window = gtk::ApplicationWindow::new(app);
    window.set_default_size(1920, 1080);
    window.set_title(Some("Transgression"));

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(r#"
    window {
      font-size: 12px;
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

    let search_button = gtk::ToggleButton::new();
    search_button.set_icon_name("system-search-symbolic");
    header_bar.pack_end(&search_button);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.set_child(Some(&container));


    // TODO: move to a separate file
    // TODO: add icons!
    let details_notebook = gtk::Notebook::builder()
        .build();

    let general_page = gtk::Grid::builder().column_spacing(10).row_spacing(10).vexpand(true).build();
    
    general_page.set_row_homogeneous(false);
    general_page.set_column_homogeneous(false);

    let name_l = gtk::Label::new(Some("Name"));
    name_l.set_halign(gtk::Align::Start);

    general_page.attach(&name_l, 0, 0, 1, 1);
    let name_label = gtk::Label::new(None);
    name_label.set_halign(gtk::Align::Start);
    general_page.attach(&name_label, 1, 0, 5, 1);
    
    details_object.property_expression("name")
       .bind(&name_label, "label", gtk::Widget::NONE);

    let size_l = gtk::Label::new(Some("Size"));
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
    seeders_l.set_halign(gtk::Align::Start);
    general_page.attach(&seeders_l, 0, 3, 1, 1);
    let seeders_label = gtk::Label::new(None);
    seeders_label.set_halign(gtk::Align::Start);
    general_page.attach(&seeders_label, 1, 3, 1, 1);
    
    details_object.property_expression("seeder-count")
       .bind(&seeders_label, "label", gtk::Widget::NONE);

    let leechers_l = gtk::Label::new(Some("Leechers"));
    leechers_l.set_halign(gtk::Align::Start);
    general_page.attach(&leechers_l, 0, 4, 1, 1);
    let leechers_label = gtk::Label::new(None);
    leechers_label.set_halign(gtk::Align::Start);
    general_page.attach(&leechers_label, 1, 4, 1, 1);
    
    details_object.property_expression("leecher-count")
       .bind(&leechers_label, "label", gtk::Widget::NONE);

    let status_l = gtk::Label::new(Some("Status"));
    status_l.set_halign(gtk::Align::Start);
    general_page.attach(&status_l, 0, 5, 1, 1);
    let status_label = gtk::Label::new(None);
    status_label.set_halign(gtk::Align::Start);
    general_page.attach(&status_label, 1, 5, 1, 1);
    
    details_object.property_expression("status")
       .bind(&status_label, "label", gtk::Widget::NONE);

    let location = gtk::Label::new(Some("Location"));
    location.set_halign(gtk::Align::Start);
    general_page.attach(&location, 0, 6, 1, 1);
    let location_label = gtk::Label::new(None);
    location_label.set_halign(gtk::Align::Start);
    general_page.attach(&location_label, 1, 6, 4, 1);
    
    details_object.property_expression("download-dir")
       .bind(&location_label, "label", gtk::Widget::NONE);

    let comment_l = gtk::Label::new(Some("Comment"));
    comment_l.set_halign(gtk::Align::Start);
    general_page.attach(&comment_l, 0, 7, 1, 1);
    let comment_label = gtk::Label::new(None);
    comment_label.set_halign(gtk::Align::Start);
    general_page.attach(&comment_label, 1, 7, 4, 1);
    
    details_object.property_expression("comment")
       .bind(&comment_label, "label", gtk::Widget::NONE);

    let hash_l = gtk::Label::new(Some("Hash"));
    hash_l.set_halign(gtk::Align::Start);
    general_page.attach(&hash_l, 0, 8, 1, 1);
    let hash_label = gtk::Label::new(None);
    hash_label.set_halign(gtk::Align::Start);
    general_page.attach(&hash_label, 1, 8, 4, 1);
    
    details_object.property_expression("hash-string")
       .bind(&hash_label, "label", gtk::Widget::NONE);

    let rate_down_label = gtk::Label::new(Some("Rate Down"));
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
    ratio_label.set_halign(gtk::Align::Start);
    general_page.attach(&ratio_label, 2, 3, 1, 1);
    let ratio_l = gtk::Label::new(None);
    ratio_l.set_halign(gtk::Align::Start);
    general_page.attach(&ratio_l, 3, 3, 1, 1);
    
    details_object.property_expression("upload-ratio")
       .bind(&ratio_l, "label", gtk::Widget::NONE);

    let ratio_limit_label = gtk::Label::new(Some("Ratio limit"));
    ratio_limit_label.set_halign(gtk::Align::Start);
    general_page.attach(&ratio_limit_label, 2, 4, 1, 1);
    let ratio_limit_l = gtk::Label::new(None);
    ratio_limit_l.set_halign(gtk::Align::Start);
    general_page.attach(&ratio_limit_l, 3, 4, 1, 1);
    
    details_object.property_expression("seed-ratio-limit")
       .bind(&ratio_limit_l, "label", gtk::Widget::NONE);

    let priority_label = gtk::Label::new(Some("Priority"));
    priority_label.set_halign(gtk::Align::Start);
    general_page.attach(&priority_label, 2, 5, 1, 1);
    let priority_l = gtk::Label::new(None);
    priority_l.set_halign(gtk::Align::Start);
    general_page.attach(&priority_l, 3, 5, 1, 1);
    
    details_object.property_expression("priority")
       .bind(&priority_l, "label", gtk::Widget::NONE);

    let completed_label = gtk::Label::new(Some("Completed"));
    completed_label.set_halign(gtk::Align::Start);
    general_page.attach(&completed_label, 4, 1, 1, 1);
    let completed_l = gtk::Label::new(None);
    completed_l.set_halign(gtk::Align::Start);
    general_page.attach(&completed_l, 5, 1, 1, 1);
    
    details_object.property_expression("percent-complete")
       .bind(&completed_l, "label", gtk::Widget::NONE);

    let downloaded_label = gtk::Label::new(Some("Downloaded"));
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
    corrupted_label.set_halign(gtk::Align::Start);
    general_page.attach(&corrupted_label, 4, 4, 1, 1);
    let corrupted_l = gtk::Label::new(None);
    corrupted_l.set_halign(gtk::Align::Start);
    general_page.attach(&corrupted_l, 5, 4, 1, 1);
    
    details_object.property_expression("corrupt-ever")
       .bind(&corrupted_l, "label", gtk::Widget::NONE);

    let completed_at_label = gtk::Label::new(Some("Completed At"));
    completed_at_label.set_halign(gtk::Align::Start);
    general_page.attach(&completed_at_label, 4, 5, 1, 1);

    let completed_at_l = gtk::Label::new(None);
    completed_at_l.set_halign(gtk::Align::Start);
    general_page.attach(&completed_at_l, 5, 5, 1, 1);
    
    details_object.property_expression("done-date")
       .bind(&completed_at_l, "label", gtk::Widget::NONE);


    details_notebook.append_page(&general_page, Some(&gtk::Label::new(Some("General"))));
    details_notebook.append_page(&gtk::Box::new(gtk::Orientation::Horizontal, 6), Some(&gtk::Label::new(Some("Trackers"))));
    details_notebook.append_page(&gtk::Box::new(gtk::Orientation::Horizontal, 6), Some(&gtk::Label::new(Some("Peers"))));
    details_notebook.append_page(&gtk::Box::new(gtk::Orientation::Horizontal, 6), Some(&gtk::Label::new(Some("Files"))));
   
    let selected_torrent_mutex : Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None));
    let selected_torrent_mutex_read = selected_torrent_mutex.clone();
    //        fucking transmission get torrents 
    {
        use std::thread;
        use transmission_client::TransmissionClient;

        
        enum TorrentUpdate {
          Full(serde_json::Value),
          Partial(serde_json::Value, serde_json::Value),
          Stats(transmission_client::SessionStats, transmission_client::FreeSpace)
        }


        

        use glib::{MainContext, PRIORITY_DEFAULT};

        let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT); 
        let (tx1, rx1) = MainContext::channel(PRIORITY_DEFAULT); 

        rx.attach(None, clone!(@weak model, @weak status_model, @weak folder_model, @weak stats => @default-return Continue(false), move |update:TorrentUpdate|{
          match update {
            TorrentUpdate::Full(xs) => {
              let torrents: Vec<TorrentInfo> = xs.as_array().unwrap()
              .iter()
              .skip(1)
              .map(|it| {
                  json_value_to_torrent_info(it)
              }).collect();
              model.splice(0, 0, &torrents);
              update_torrent_stats(&model, &status_model, &folder_model);
                // mutex is not needed as we do this in the ui thread 
                // *group_stats_mutex.lock().unwrap() = group_stats;
              },
           TorrentUpdate::Partial(xs, removed) => {
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
                     update_torrent_stats(&model, &status_model, &folder_model);
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

        rx1.attach(None, clone!(@strong details_object => @default-return Continue(false), move |details:TorrentDetails|{
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
         //   println!("{:#?}", details);
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
                tx.send(TorrentUpdate::Partial(torrents, removed)).expect("blah");
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

    let directory_filter = gtk::CustomFilter::new(move |_|  true);
    let status_filter    = gtk::CustomFilter::new(move |_| true);

    let main_filter = gtk::EveryFilter::new();
    main_filter.append(&name_filter);
    main_filter.append(&directory_filter);
    main_filter.append(&status_filter);
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

    let status_factory = gtk::SignalListItemFactory::new();

    status_factory.connect_setup(move |_, list_item| {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        list_item.set_child(Some(&hbox));

        let name_label = gtk::Label::new(None);
        let count_label = gtk::Label::new(None);
        hbox.append(&name_label);
        hbox.append(&count_label);

        list_item
            .property_expression("item")
            .chain_property::<StatusInfo>("count")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<glib::Object>, count: u64| {
                format!("({})", count)
            }))
            .bind(&count_label, "label", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<StatusInfo>("id")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<glib::Object>, id: i64| {
              match id {
                STOPPED => "Paused",
                VERIFY_QUEUED => "Checking queued",
                VERIFYING => "Checking", 
                DOWN_QUEUED => "Download queued", 
                DOWNLOADING => "Downloading", 
                SEED_QUEUED => "Seeding queued", 
                SEEDING => "Seeding", 
                ALL => "All",
                _   => "???"
              } 
            }))
            .bind(&name_label, "label", gtk::Widget::NONE);
    });

    let status_selection_model = gtk::SingleSelection::new(Some(&status_model));

    status_selection_model.connect_selected_item_notify(clone!(@weak directory_filter => move |s| {
      match s.selected_item() {
          Some(item) => {
            let id = item.property_value("id").get::<i64>().unwrap();   
            if id == -1 {
              status_filter.set_filter_func(|_| true);
            } else {
                status_filter.set_filter_func(move |item| {
                    item.property_value("status").get::<i64>().unwrap() == id
                });
            }
          },
          None       => {
            status_filter.set_filter_func(|_| true);
          }
      }
    }));
    let status_view = gtk::ListView::new(Some(&status_selection_model), Some(&status_factory));

    // todo: use left align, add margin..

    let folders_factory = gtk::SignalListItemFactory::new();
    folders_factory.connect_setup(move |_, list_item| {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        list_item.set_child(Some(&hbox));

        let name_label = gtk::Label::new(None);
        let count_label = gtk::Label::new(None);
        hbox.append(&name_label);
        hbox.append(&count_label);

        list_item
            .property_expression("item")
            .chain_property::<FolderInfo>("count")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<glib::Object>, count: u64| {
                if count !=  NO_SHOW {
                  format!("({})", count)
                } else {
                    "".to_string()
                }
            }))
            .bind(&count_label, "label", gtk::Widget::NONE);

        list_item
            .property_expression("item")
            .chain_property::<FolderInfo>("name")
            .chain_closure::<String>(gtk::glib::closure!(|_: Option<glib::Object>, name: String| {
                process_folder(name)
            }))
            .bind(&name_label, "label", gtk::Widget::NONE);
    });

    let folder_selection_model = gtk::SingleSelection::new(Some(&folder_model));
 //   folder_selection_model.set_autoselect(true);
//    folder_selection_model.set_can_unselect(false);
    let folder_view = gtk::ListView::new(Some(&folder_selection_model), Some(&folders_factory));
    folder_view.set_vexpand(true);


    folder_selection_model.connect_selected_item_notify(clone!(@weak directory_filter => move |s| {
      match s.selected_item() {
          Some(item) => {
            let name = item.property_value("name").get::<String>().unwrap();   
            if name == "any" {
              directory_filter.set_filter_func(|_| true);
            } else {
              directory_filter.set_filter_func(move |item| {
                  item.property_value("download-dir").get::<String>().unwrap() == name
              });
            }
          },
          None       => {
            directory_filter.set_filter_func(|_| true);
          }
      }
    }));
//    folder_selection_model.connect_selection_changed(move |_self, position, _| {
//      if _self.connect_selected_item_notify();
//    });

    // separator 
    // dynamic list of labels 
    left_pane.append(&status_view);
    left_pane.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    left_pane.append(&folder_view);

    // ========================================================================

    main_view.set_valign(gtk::Align::Fill);
    main_view.set_start_child(&left_pane);
    main_view.set_end_child(&right_hbox);
    main_view.set_resize_end_child(true);
    main_view.set_resize_start_child(false);
    main_view.set_shrink_start_child(true);

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



//fn build_sidebar(parent: gtk::Paned) {

//}


