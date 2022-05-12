//use std::fmt;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub downloaded: u64,
    pub children: Vec<Node>
}

pub fn format_size(i: i64) -> String {
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
pub fn format_download_speed(i: i64) -> String { 
                if i == 0 {
                  "".to_string()
                } else if i > 1024*1024 {
                  format!("{} Mb/s", i / (1024 * 1024))
                } else {
                  format!("{} Kb/s", i / 1024)
                }
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

fn group(xs: Vec<Vec<String>>) -> Vec<(String, Vec<Vec<String>>)> {
    if xs.len() == 0 {
        return vec![];
    }
    let mut curr = String::from("");
    let mut ys: Vec<Vec<String>> = vec![];
    let mut res: Vec<(String, Vec<Vec<String>>)> = vec![];

    for mut x in xs {
        let head = x.remove(0);
        if head != curr {
          if curr != "" {
            ys.retain(|x| x.len() > 0);
            res.push((curr, ys));
          } 
          curr = head;
          ys = vec![x];
        } else {
          ys.push(x);
        }
    }
    ys.retain(|x| x.len() > 0);
    res.push((curr, ys));

    res
}

pub fn build_tree(parent_path: &String, mut xs: Vec<Vec<String>>) -> (u64, u64, Vec<Node>) {
    xs.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
    let xxs = group(xs);
    let mut ns: Vec<Node> = vec![];
    for (parent, ys) in xxs {
          let path = if parent_path == "" { parent.to_string() } else { format!("{}/{}", parent_path, parent) };
          ns.push(Node { name: parent.to_string(), path: path.clone(), children: build_tree(&path, ys)});
    }
    ns
}

