mod command_processor;
mod config;
mod create_torrent_dialog;
mod file_table;
mod magnet_tools;
mod notification_utils;
mod objects;
mod utils;
use crate::command_processor::{CommandProcessor, TorrentCmd};
use crate::config::get_or_create_config;
use crate::file_table::{build_bottom_files, create_file_model};
use crate::objects::FileObject;
use create_torrent_dialog::add_torrent_dialog2;
use transg::transmission;

use glib::clone;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use std::cell::{Cell, RefCell};
use std::env;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn main() {
    gio::resources_register_include!("transgression.gresource").expect("Failed to register resources.");

    let app = gtk::Application::new(
        Some("org.transgression.TransgressionOpenMagnet"),
        gio::ApplicationFlags::HANDLES_COMMAND_LINE,
    );

    app.connect_command_line(|app, _| {
        app.activate();
        0
    });

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let config = Arc::new(Mutex::new(get_or_create_config()));
    let args: Vec<String> = env::args().collect();
    let magnet_url: String = if args.len() != 2 {
        println!("Usage: transgression-open-magnet '<magnet-url>'");
        app.quit();
        "".to_string()
    } else {
        args[1].clone()
    };

    let window = gtk::ApplicationWindow::new(app);

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_resource("/org/transgression/ui.css");

    gtk::StyleContext::add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    let (mut processor, _) = CommandProcessor::create();
    let sender = processor.get_sender();
    processor.run(config.clone(), false, true);

    window.set_title(Some("Transgression Open Magnet"));

    //   let sender = tx2.clone();
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
    vbox.add_css_class("dialog-container");

    let _cancel_fetch = Rc::new(Cell::new(false));
    let _magnet_done = Rc::new(Cell::new(false));
    let magnet_data = Rc::new(RefCell::new(None));
    let destination = gtk::DropDown::builder().build();
    let start_paused_checkbox = gtk::CheckButton::builder().active(false).label("Start paused").build();

    add_torrent_dialog2(
        &vbox,
        &Some(magnet_url.clone()),
        &None,
        _cancel_fetch.clone(),
        _magnet_done.clone(),
        magnet_data.clone(),
        &destination,
        &start_paused_checkbox,
        config.clone()
    );

    let bottom_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    bottom_box.set_halign(gtk::Align::End);
    vbox.append(&bottom_box);
    let ok_button = gtk::Button::new();
    ok_button.set_label("Ok");
    ok_button.add_css_class("suggested-action");
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
         sender.blocking_send(TorrentCmd::AddTorrent(folder, None, Some(metainfo), start_paused)).expect("failure snd move");
       } else {
         sender.blocking_send(TorrentCmd::AddTorrent(folder, Some(magnet_url.to_string()), None, start_paused)).expect("failure snd move");
       }
     std::thread::sleep(std::time::Duration::from_millis(300));
     app.quit();
  }));

  window.present();
}
