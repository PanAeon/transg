//use std::fmt;
use transg::transmission;
use chrono::{UTC, NaiveDateTime, DateTime};

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub downloaded: u64,
    pub children: Vec<Node>
}

const BYTES_TB: i64 = 1024*1024*1024*1024; 
const BYTES_GB: i64 = 1024*1024*1024; 
const BYTES_MB: i64 = 1024*1024; 
const F_BYTES_TB: f64 = 1024.0*1024.0*1024.0*1024.0; 
const F_BYTES_GB: f64 = 1024.0*1024.0*1024.0; 
const F_BYTES_MB: f64 = 1024.0*1024.0; 

pub fn format_size(i: i64) -> String {
                if i == 0 {
                  "".to_string()
                } else if i > BYTES_TB {
                  format!("{:.2} Tib", i as f64 / F_BYTES_TB )
                } else if i > BYTES_GB {
                  format!("{:.2} Gib", i as f64 / F_BYTES_GB)
                } else if i > BYTES_MB {
                  format!("{:.2} Mib", i as f64 / F_BYTES_MB)
                } else {
                  format!("{:.2} Kib", i as f64 / 1024.0)
                }
}
pub fn format_download_speed(i: i64) -> String { 
                if i == 0 {
                  "".to_string()
                } else if i > BYTES_MB {
                  format!("{:.2} Mib/s", i as f64 / F_BYTES_MB)
                } else {
                  format!("{:.2} Kib/s", i as f64 / 1024.0)
                }
}

pub fn format_time(i: u64) -> String {
  let naive = NaiveDateTime::from_timestamp(i.try_into().expect("can't convert from u64 into i64"), 0);
  let datetime: DateTime<UTC> = DateTime::from_utc(naive, UTC);
  datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_eta(secs: i64) -> String {
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


pub fn do_build_tree(parent_path: &str, level: usize, xs: Vec<(u64, u64, Vec<String>)> ) -> Vec<Node> {
    let mut ns: Vec<Node> = vec![];

    let mut parents : Vec<String> = xs.iter().filter(|x| x.2.len() > level).map(|x| x.2[level].clone()).collect();
    parents.sort();
    parents.dedup();

    for name in parents {
        let children : Vec<(u64, u64, Vec<String>)> = xs.iter().filter(|x| x.2.len() > level && x.2[level] == name).map(|x| x.clone()).collect();
        let path = if parent_path == "" { name.to_string() } else { format!("{}/{}", parent_path, name) };
        let size = children.iter().map(|x| x.0).sum();
        let downloaded = children.iter().map(|x| x.1).sum();
        let cs = if children.len() > 1 {
          do_build_tree(&path, level+1, children)
        } else {
            vec![]
        };
        ns.push(Node { 
              name, 
              path, 
              children: cs,
              size,
              downloaded
          });
    }
    ns
}
pub fn build_tree(files: &Vec<transmission::File> ) -> Vec<Node> {
    let mut xs : Vec<(u64, u64, Vec<String>)> = files.iter().map(|f| (f.length, f.bytes_completed, f.name.split('/').map(|x| String::from(x)).collect())).collect();
    xs.sort_by(|a, b| a.2[0].partial_cmp(&b.2[0]).unwrap());
    do_build_tree("", 0, xs)
}


