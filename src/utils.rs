//use std::fmt;

#[derive(Debug, Clone)]
pub struct Node {
    pub data: String,
    pub path: String,
    pub children: Vec<Node>
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

pub fn build_tree(parent_path: &String, mut xs: Vec<Vec<String>>) -> Vec<Node> {
    xs.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
    let xxs = group(xs);
    let mut ns: Vec<Node> = vec![];
    for (parent, ys) in xxs {
          let path = if parent_path == "" { parent.to_string() } else { format!("{}/{}", parent_path, parent) };
          ns.push(Node { data: parent.to_string(), path: path.clone(), children: build_tree(&path, ys)});
    }
    ns
}

