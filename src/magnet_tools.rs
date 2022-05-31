use libtorrent_sys::ffi::*;

use std::time::Duration;
use async_std::task;
use std::cell::Cell;
use std::rc::Rc;
use tempdir::TempDir;

pub async fn magnet_to_metainfo(uri: &str, cancelation: Rc<Cell<bool>>) -> Option<Vec<u8>> {
    let mut session = lt_create_session();
    let tmp_dir = TempDir::new("transg").expect("can't create temporary directory");
    let s = tmp_dir.path().to_str().expect("foobar");
    let mut torrent_param = lt_parse_magnet_uri(&uri, &s);
    let hdl = lt_session_add_torrent(session.pin_mut(), torrent_param.pin_mut());

    loop {
    if cancelation.get() {
	    lt_session_pause(session.pin_mut());
	    lt_session_remove_torrent(session.pin_mut(), &hdl);
        return None;
    }

    println!("... fetching metadata");
	if lt_torrent_has_metadata(&hdl) {
        println!("... got metadata");
	    lt_session_pause(session.pin_mut());
	    //let torrent_name = lt_torrent_get_name(&hdl);

	    let bin = lt_torrent_bencode(&hdl);
	    lt_session_remove_torrent(session.pin_mut(), &hdl);

	    return Some(bin.to_vec()); 
	}


	task::sleep(Duration::from_millis(1000)).await;
    }
}
