use gtk::gio;

fn main() {
    println!("cargo:rerun-if-changed=src/resources");
    gio::compile_resources(
        "src/resources",
        "src/resources/resources.gresource.xml",
        "transgression.gresource",
    );
}

