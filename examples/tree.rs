
use gtk::prelude::*;

fn main() {
    let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some("First GTK Program"));
    window.set_default_size(350, 70);

    let button = gtk::Button::with_label("Click me!");

    let model = gtk::TreeStore::new(&[String::static_type(), u64::static_type(), f64::static_type(), bool::static_type(), u8::static_type()]);

    window.set_child(Some(&button));

    window.show();
}
