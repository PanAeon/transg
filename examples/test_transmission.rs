use transmission::{Client, ClientConfig};

fn main() {
    let c = ClientConfig::new()
        .app_name("testing")
        .config_dir("/tmp/foo")
        .download_dir("/tmp/bar");
    let mut c = Client::new(c);

    let m = "magnet:?xt=urn:btih:95E5111D863B1CA7F9CED8063A430A4ADAF3B3E9&tr=http%3A%2F%2Fbt.t-ru.org%2Fann%3Fmagnet&dn=%D0%96%D0%B8%D0%BB%D0%B0-%D0%B1%D1%8B%D0%BB%D0%B0%20%D0%B4%D0%B5%D0%B2%D0%BE%D1%87%D0%BA%D0%B0%20(%D0%92%D0%B8%D0%BA%D1%82%D0%BE%D1%80%20%D0%AD%D0%B9%D1%81%D1%8B%D0%BC%D0%BE%D0%BD%D1%82)%20%5B1944%2C%20%D0%94%D0%B5%D1%82%D1%81%D0%BA%D0%B8%D0%B9%2C%20%D0%B2%D0%BE%D0%B5%D0%BD%D0%BD%D0%B0%D1%8F%20%D0%B4%D1%80%D0%B0%D0%BC%D0%B0%2C%20DVDRip%5D";
    let t = c.add_torrent_magnet(m).unwrap();
    t.start();

    // Run until done
    while t.stats().percent_complete < 1.0 {
        print!("{:#?}\r", t.stats().percent_complete);
    }
    c.close();
}
