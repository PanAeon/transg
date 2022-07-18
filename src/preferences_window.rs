use std::rc::Rc;

use gtk::prelude::*;
use crate::config;
use config::Config;
// hmm, can show dialog instead
pub async fn preferences_dialog<W: IsA<gtk::Window>>(
    window: Rc<W>
) {
    let cfg = config::get_or_create_config(); 
    let dialog = gtk::Dialog::builder()
        .transient_for(&*window)
        .modal(true)
        .default_width(960)
        .default_height(840)
        .use_header_bar(1)
        .title("Preferences")
        .name("preferences-window")
        .build();

    dialog.add_css_class("simple-dialog");
    //dialog.header_bar()
    dialog.add_button("Close", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Ok);

    // TODO: add custom widget for dropdown, so user can enter custom path, maybe directory picker?
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    vbox.set_margin_start(0);

    let tabs = gtk::Notebook::builder()
        .tab_pos(gtk::PositionType::Top)
        .build();

    tabs.set_margin_top(0);

    tabs.append_page(&connection_page(&cfg), Some(&gtk::Image::builder().icon_name("network-idle").build()));
    tabs.append_page(&general_settings_page(&cfg), Some(&gtk::Image::builder().icon_name("settings-symbolic").build()));
    tabs.append_page(&folder_page(&cfg), Some(&gtk::Image::builder().icon_name("folder-symbolic").build()));
    
    vbox.append(&tabs);
    dialog.content_area().append(&vbox); // and a label for destination))
    let response = dialog.run_future().await;
    dialog.close();
    if response == gtk::ResponseType::Ok {
    }
}

fn connection_page(cfg: &Config) -> gtk::ScrolledWindow {
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_start(20)
        .margin_top(20)
        .margin_bottom(20)
        .margin_end(20)
        .spacing(5)
        .build();
    let window = gtk::ScrolledWindow::builder()
        .min_content_width(360)
        .vexpand(true)
        .child(&vbox)
        .build();
    let l = gtk::Label::builder().label("Name").halign(gtk::Align::Start).build();
    let i = gtk::Entry::builder().text("NAS").build();
    vbox.append(&l);
    vbox.append(&i);
    let l = gtk::Label::builder().label("Url").halign(gtk::Align::Start).build();
    let i = gtk::Entry::builder().text(&cfg.connection_string).build();
    vbox.append(&l);
    vbox.append(&i);

    let s = "Prererences are not quiet ready yet. You'll have to edit ~/.config/transg/config.json directly..";
    let l = gtk::Label::builder().margin_top(20).label(&s).halign(gtk::Align::Start).build();
    vbox.append(&l);

    window
}

fn general_settings_page(cfg: &Config) -> gtk::ScrolledWindow {
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_start(20)
        .margin_top(20)
        .margin_bottom(20)
        .margin_end(20)
        .spacing(5)
        .build();
    let window = gtk::ScrolledWindow::builder()
        .min_content_width(360)
        .vexpand(true)
        .child(&vbox)
        .build();
    let l = gtk::Label::builder().label("Remote base dir").halign(gtk::Align::Start).build();
    let i = gtk::Entry::builder().text(&cfg.remote_base_dir).build();
    vbox.append(&l);
    vbox.append(&i);
    let l = gtk::Label::builder().label("Local base dir").halign(gtk::Align::Start).build();
    let i = gtk::Entry::builder().text(&cfg.local_base_dir).build();
    vbox.append(&l);
    vbox.append(&i);

    window
}

fn folder_page(cfg: &Config) -> gtk::ScrolledWindow {
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_start(20)
        .margin_top(20)
        .margin_bottom(20)
        .margin_end(20)
        .spacing(5)
        .build();
    let window = gtk::ScrolledWindow::builder()
        .min_content_width(360)
        .vexpand(true)
        .child(&vbox)
        .build();
    for f in &cfg.directories {
      let i = gtk::Entry::builder().text(&f).build();
      i.set_margin_bottom(15);
      vbox.append(&i);
    }

    window
}
