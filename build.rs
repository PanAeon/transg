use gtk::gio;

fn main() {
    println!("cargo:rerun-if-changed=src/resources");
    //println!("cargo:rustc-link-lib=dylib=torrent-rasterbar");
    //println!("cargo:rustc-link-search=native=/nix/store/j4pf0kdrfqlajl9x6ym3vlayg0787i6c-libtorrent-rasterbar-2.0.4/lib/");
    //println!("cargo:rustc-link-search=native=/nix/store/4xm2jnpkx5na1prkjaahx9x1sd34kyqa-libtorrent-rasterbar-1.1.11/lib/");
    gio::compile_resources(
        "src/resources",
        "src/resources/resources.gresource.xml",
        "transgression.gresource",
    );
}
