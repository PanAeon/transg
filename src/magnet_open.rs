mod objects;
mod utils;
mod torrent_details_grid;
mod magnet_tools;
mod notification_utils;
mod file_table;
mod create_torrent_dialog;
use create_torrent_dialog::add_torrent_dialog2;
use crate::objects::FileObject;
use transg::transmission;
use notification_utils::notify;
use crate::file_table::{create_file_model, build_bottom_files};
 
use gtk::prelude::*;
use gtk::Application;
use glib::clone;
use gtk::glib;
use gtk::gio;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::env;

        use transmission::TransmissionClient;

        #[derive(Debug)]
        pub enum TorrentCmd {
            AddTorrent(Option<String>, Option<String>, Option<String>, bool) // download dir, filename, metainfo, start_paused
        }


fn main() {
    gio::resources_register_include!("transgression.gresource")
        .expect("Failed to register resources.");

    let app = gtk::Application::new(
        Some("org.transgression.TransgressionOpenMagnet"),
        gio::ApplicationFlags::HANDLES_COMMAND_LINE);

    app.connect_command_line(|app, _| {
      app.activate();
      0
    });

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {


    let args: Vec<String> = env::args().collect();
    let magnet_url : String = if args.len() != 2 {
        println!("Usage: transgression-open-magnet '<magnet-url>'");
        app.quit();
        "".to_string()
    } else {
       args[1].clone() 
    };


    let window = gtk::ApplicationWindow::new(app);

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(r#"
    window {
      font-size: 14px;
      padding: 20px 20px 20px 20px;
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







        let client = TransmissionClient::new(&"http://192.168.1.217:9091/transmission/rpc".to_string());
        let (sender, rx2) = mpsc::channel(); 


        thread::spawn(move || {
          use tokio::runtime::Runtime;
          let rt = Runtime::new().expect("create tokio runtime");


          
          loop {
            let cmd = rx2.recv().expect("probably ticker thread panicked");

          rt.block_on(async {
            match cmd {
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
                        let _ =  notify("Torrent Added!", "").await; // TODO: add name
                    } else {
                        let _ = notify("Error!", "").await;
                      println!("{:?}", res);
                    }
                } 
            }
          });

                  
          }       
          });
   
 
  window.set_title(Some("Transgression Open Magnet"));
   
 //   let sender = tx2.clone();
  let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
  let _cancel_fetch = Rc::new(Cell::new(false));
  let _magnet_done = Rc::new(Cell::new(false));
  let magnet_data =  Rc::new(RefCell::new(None));
  let destination = gtk::DropDown::builder().build();
  let start_paused_checkbox = gtk::CheckButton::builder().active(false).label("Start paused").build(); 


  add_torrent_dialog2(&vbox, &Some(magnet_url.clone()), &None, _cancel_fetch.clone(), _magnet_done.clone(), magnet_data.clone(), &destination, &start_paused_checkbox);



  let bottom_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  bottom_box.set_halign(gtk::Align::End);
  vbox.append(&bottom_box);
  let ok_button = gtk::Button::new();
  ok_button.set_label("Ok");
  bottom_box.append(&ok_button);
  let cancel_button = gtk::Button::new();
  cancel_button.set_label("Cancel");
  bottom_box.append(&cancel_button);

  window.set_child(Some(&vbox)); 

  cancel_button.connect_clicked(clone!(@weak app => move |_| {
      app.quit();
  }));
  ok_button.connect_clicked(clone!(@weak app => move |_| {
     let start_paused =start_paused_checkbox.is_active();
     let folder = destination.selected_item().map(|x| x.property_value("string").get::<String>().expect("should be string")); 
     _cancel_fetch.set(true); 
     if let Some(data) = &*magnet_data.borrow() {
         let metainfo = base64::encode(data);
         sender.send(TorrentCmd::AddTorrent(folder, None, Some(metainfo), start_paused)).expect("failure snd move");
       } else { 
         sender.send(TorrentCmd::AddTorrent(folder, Some(magnet_url.to_string()), None, start_paused)).expect("failure snd move");
       }
     thread::sleep(std::time::Duration::from_millis(300));
     app.quit();
  }));

  window.present();
}

