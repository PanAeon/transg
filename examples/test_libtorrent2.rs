use libtorrent_sys::ffi::*;

use std::fs::File;
use std::io::{self, Write};
use std::{thread, time::Duration};

fn main() {
    let uri = std::env::args().nth(1).expect("no pattern given");
    let mut session = lt_create_session();
    let mut torrent_param = lt_parse_magnet_uri(&uri);
    let hdl = lt_session_add_torrent(session.pin_mut(), torrent_param.pin_mut());

    loop {
        if lt_torrent_has_metadata(&hdl) {
            lt_session_pause(session.pin_mut());
            let torrent_name = lt_torrent_get_name(&hdl);
            println!("\ncreate file: {}.torrent", torrent_name);
            io::stdout().flush().unwrap();

            let bin = lt_torrent_bencode(&hdl);
            let mut ofile = File::create(format!("{}.torrent", torrent_name)).expect("unable to create file");
            ofile.write_all(&bin).expect("unable to write");

            lt_session_remove_torrent(session.pin_mut(), &hdl);
            break;
        }

        print!(".");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(1000));
    }
}
