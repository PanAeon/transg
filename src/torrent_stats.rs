use std::collections::HashMap;
use gtk::prelude::*;
use gtk::gio::ListStore;
use crate::CategoryObject;
#[derive(Debug)]
pub struct TorrentGroupStats {
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

pub fn empty_group_stats() -> TorrentGroupStats {
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

pub fn process_folder(s: String) -> String {
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


pub fn update_torrent_stats(model: &ListStore, category_model: &ListStore) {
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
