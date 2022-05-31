use transg::libtorrent::*;
use std::mem;


fn main() {
  println!("god help us..");
  unsafe {
    let session = session_create(tags_SES_LISTENPORT as i32, 6881 as i32,
                                 tags_SES_LISTENPORT_END, 6889 as i32,
                                 tags_TAG_END);
    let t = session_add_torrent(session,
                                tags_TOR_FILENAME as i32, b"/home/vitalii/lab/foo.torrent\0".as_ptr() as *const i8,
                                tags_TOR_SAVE_PATH, b"/tmp\0".as_ptr() as *const i8,
                                tags_TAG_END);
    if t < 0 {
        println!("something went wrong!");
        std::process::exit(-1);
    }
    let torrent_status : *mut torrent_status = std::ptr::null_mut(); 
    println!("press CTRL-C to stop");
    loop {
      if torrent_get_status(t, torrent_status, mem::size_of::<torrent_status>() as i32) < 0 { break; }
      println!("{:?}", torrent_status);
    }
    session_close(session);
  }
  println!("the world is safe yet");
}
