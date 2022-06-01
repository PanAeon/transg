use gtk::prelude::*;
use crate::FileObject;
use gtk::gio::ListStore;
use gtk::glib;
use crate::utils::{format_size, Node};
use std::cell::RefCell;
use std::rc::Rc;
use gtk::TreeListRow;

pub fn create_file_model(tree: &Rc<RefCell<Vec<Node>>>)  ->  gtk::TreeListModel 
  {

    let m0 = ListStore::new(FileObject::static_type());
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
            let m0 = ListStore::new(FileObject::static_type());
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

pub fn get_children(tree: &Vec<Node>, path: String) -> Vec<Node> {
  fn do_get_children(tree: &Vec<Node>, mut path: Vec<&str>) -> Vec<Node> {
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



pub fn build_bottom_files(file_table: &gtk::ColumnView, include_progress: bool) -> gtk::ScrolledWindow {

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

