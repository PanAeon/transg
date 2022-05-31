use glib::Object;
use gtk::glib;
//use gtk::prelude::*;
//use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct TorrentDetailsObject(ObjectSubclass<imp::TorrentDetailsObject>);
}

impl TorrentDetailsObject {
    pub fn new(
        id: &u64,
        name: &String,
        eta: &i64,
        size_when_done: &u64,
        seeder_count: &i64,
        leecher_count: &i64,
        status: &u64,
        download_dir: &String,
        comment: &String,
        hash_string: &String,
        rate_download: &u64,
        rate_upload: &u64,
        upload_ratio: &f64,
        seed_ratio_limit: &u64,
        priority: &u64,
        done_date: &u64,
        percent_complete: &f64,
        downloaded_ever: &u64,
        uploaded_ever: &u64,
        corrupt_ever: &u64,
//        labels: &Vec<String>,
        piece_count: &u64,
        pieces: &String,
        error: &i64,
        error_string: &String,
      //  files: &Vec<FileObject>,
      //  file_stats: &Vec<FileStatsObject>,
      //  priorities: &Vec<u8>,
      //  wanted: &Vec<u8>,
      //  peers: &Vec<PeerObject>,
      //  trackers: &Vec<TrackerObject>,
      //  tracker_stats: &Vec<TrackerStatsObject>,
    ) -> Self {
        Object::new(&[
            ("id", &id),
            ("name", &name),
            ("eta", &eta),
            ("size-when-done", &size_when_done),
            ("seeder-count", &seeder_count),
            ("leecher-count", &leecher_count),
            ("status", &status),
            ("download-dir", &download_dir),
            ("comment", &comment),
            ("hash-string", &hash_string),
            ("rate-download", &rate_download),
            ("rate-upload", &rate_upload),
            ("upload-ratio", &upload_ratio),
            ("seed-ratio-limit", &seed_ratio_limit),
            ("priority", &priority),
            ("done-date", &done_date),
            ("percent-complete", &percent_complete),
            ("downloaded-ever", &downloaded_ever),
            ("uploaded-ever", &uploaded_ever),
            ("corrupt-ever", &corrupt_ever),
 //           ("labels", &labels),
            ("piece-count", &piece_count),
            ("pieces", &pieces),
            ("error", &error),
            ("error-string", &error_string)
          //  ("files", &files),
          //  ("file-stats", &file_stats),
          //  ("priorities", &priorities),
          //  ("wanted", &wanted),
          //  ("peers", &peers),
          //  ("trackers", &trackers),
          //  ("tracker-stats", &tracker_stats),
        ])
        .expect("Failed to create 'TorrentDetailsObject'")
    }
}
impl Default for TorrentDetailsObject {
    fn default() -> Self {
        Self::new(&0, &String::from("shit"), &0, &0, &0, &0, &0, &String::from(""), &String::from(""), &String::from(""), &0, &0,
          &0.0, &0, &0, &0, &0.0, &0, &0, &0, &0, &String::from(""), &0, &String::from(""))
    }
}

mod imp {

use gtk::glib::ParamSpecDouble;
use once_cell::sync::Lazy;
use std::cell::{RefCell, Cell};
use glib::{ParamFlags, ParamSpec, ParamSpecString, ParamSpecInt64, ParamSpecUInt64, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct TorrentDetailsObject {
        id: Cell<u64>,
        name: RefCell<String>,
        eta: Cell<i64>,
        size_when_done: Cell<u64>,
        seeder_count: Cell<i64>,
        leecher_count: Cell<i64>,
        status: Cell<u64>,
        download_dir: RefCell<String>,
        comment: RefCell<String>,
        hash_string: RefCell<String>,
        rate_download: Cell<u64>,
        rate_upload: Cell<u64>,
        upload_ratio: Cell<f64>,
        seed_ratio_limit: Cell<u64>,
        priority: Cell<u64>,
        done_date: Cell<u64>,
        percent_complete: Cell<f64>,
        downloaded_ever: Cell<u64>,
        uploaded_ever: Cell<u64>,
        corrupt_ever: Cell<u64>,
        //labels: RefCell<Vec<String>>,
        piece_count: Cell<u64>,
        pieces: RefCell<String>,
        error: Cell<i64>,
        error_string: RefCell<String>,
     //   files: RefCell<Vec<FileObject>>,
     //   file_stats: RefCell<Vec<FileStatsObject>>,
     //   priorities: RefCell<Vec<u8>>,
     //   wanted: RefCell<Vec<u8>>,
     //   peers: RefCell<Vec<PeerObject>>,
     //   trackers: RefCell<Vec<TrackerObject>>,
     //   tracker_stats: RefCell<Vec<TrackerStatsObject>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for TorrentDetailsObject {
        const NAME: &'static str = "TorrentDetailsObject";
        type Type = super::TorrentDetailsObject;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for TorrentDetailsObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                  ParamSpecUInt64::new( "id", "id", "id", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecString::new( "name", "name", "name", None, ParamFlags::READWRITE,),
ParamSpecInt64::new( "eta", "eta", "eta", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "size-when-done", "size-when-done", "size-when-done", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecInt64::new( "seeder-count", "seeder-count", "seeder-count", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecInt64::new( "leecher-count", "leecher-count", "leecher-count", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "status", "status", "status", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecString::new( "download-dir", "download-dir", "download-dir", None, ParamFlags::READWRITE,),
ParamSpecString::new( "comment", "comment", "comment", None, ParamFlags::READWRITE,),
ParamSpecString::new( "hash-string", "hash-string", "hash-string", None, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "rate-download", "rate-download", "rate-download", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "rate-upload", "rate-upload", "rate-upload", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecDouble::new( "upload-ratio", "upload-ratio", "upload-ratio", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "seed-ratio-limit", "seed-ratio-limit", "seed-ratio-limit", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "priority", "priority", "priority", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "done-date", "done-date", "done-date", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecDouble::new( "percent-complete", "percent-complete", "percent-complete", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "downloaded-ever", "downloaded-ever", "downloaded-ever", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "uploaded-ever", "uploaded-ever", "uploaded-ever", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "corrupt-ever", "corrupt-ever", "corrupt-ever", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
//ParamSpecValueArray::new("labels", "labels", "labels", &ParamSpecString::new("s", "s", "s", None, ParamFlags::READWRITE), ParamFlags::READWRITE,),
ParamSpecUInt64::new( "piece-count", "piece-count", "piece-count", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecString::new( "pieces", "pieces", "pieces", None, ParamFlags::READWRITE,),
ParamSpecInt64::new( "error", "error", "error", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecString::new( "error-string", "error-string", "error-string", None, ParamFlags::READWRITE,),
//ParamSpecValueArray::new("files", "files", "files", FileObject::static_type(), ParamFlags::READWRITE,),
//ParamSpecValueArray::new("file-stats", "file-stats", "file-stats", FileStatsObject::static_type(), ParamFlags::READWRITE,),
//ParamSpecValueArray::new("priorities", "priorities", "priorities", ParamSpecUChar::new("c", "c", "c", u8::MIN, u8::MAX, 0, ParamFlags::READWRITE), ParamFlags::READWRITE,),
//ParamSpecValueArray::new("wanted", "wanted", "wanted", ParamSpecUChar::new("c", "c", "c", u8::MIN, u8::MAX, 0, ParamFlags::READWRITE), ParamFlags::READWRITE,),
//ParamSpecValueArray::new("peers", "peers", "peers", PeerObject::static_type(), ParamFlags::READWRITE,),
//ParamSpecValueArray::new("trackers", "trackers", "trackers", TrackerObject::static_type(), ParamFlags::READWRITE,),
//ParamSpecValueArray::new("tracker-stats", "tracker-stats", "tracker-stats", TrackerStatsObject::static_type(), ParamFlags::READWRITE,),    
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" =>   { self.id.replace(value.get().expect("The value needs to be of type `u64`."));},
"name" => {  self.name.replace(value.get().expect("The value needs to be of type `String`."));},
"eta" =>  {  self.eta.replace(value.get().expect("The value needs to be of type `i64`."));},
"size-when-done" => {   self.size_when_done.replace(value.get().expect("The value needs to be of type `u64`."));},
"seeder-count" => {   self.seeder_count.replace(value.get().expect("The value needs to be of type `i64`."));},
"leecher-count" =>  {  self.leecher_count.replace(value.get().expect("The value needs to be of type `i64`."));},
"status" => {   self.status.replace(value.get().expect("The value needs to be of type `u64`."));},
"download-dir" => {   self.download_dir.replace(value.get().expect("The value needs to be of type `String`."));},
"comment" =>  {  self.comment.replace(value.get().expect("The value needs to be of type `String`."));},
"hash-string" => {   self.hash_string.replace(value.get().expect("The value needs to be of type `String`."));},
"rate-download" =>  {  self.rate_download.replace(value.get().expect("The value needs to be of type `u64`."));},
"rate-upload" =>  {  self.rate_upload.replace(value.get().expect("The value needs to be of type `u64`."));},
"upload-ratio" =>  {  self.upload_ratio.replace(value.get().expect("The value needs to be of type `f64`."));},
"seed-ratio-limit" => {   self.seed_ratio_limit.replace(value.get().expect("The value needs to be of type `u64`."));},
"priority" =>  {  self.priority.replace(value.get().expect("The value needs to be of type `u64`."));},
"done-date" => {   self.done_date.replace(value.get().expect("The value needs to be of type `u64`."));},
"percent-complete" =>  {  self.percent_complete.replace(value.get().expect("The value needs to be of type `f64`."));},
"downloaded-ever" =>  {  self.downloaded_ever.replace(value.get().expect("The value needs to be of type `u64`."));},
"uploaded-ever" =>  {  self.uploaded_ever.replace(value.get().expect("The value needs to be of type `u64`."));},
"corrupt-ever" =>  {  self.corrupt_ever.replace(value.get().expect("The value needs to be of type `u64`."));},
//"labels" =>  {  self.labels.replace(value.get().expect("The value needs to be of type `Vec<String>`."));},
"piece-count" => {   self.piece_count.replace(value.get().expect("The value needs to be of type `u64`."));},
"pieces" =>  {  self.pieces.replace(value.get().expect("The value needs to be of type `String`."));},
"error" =>  {  self.error.replace(value.get().expect("The value needs to be of type `String`."));},
"error-string" =>  {  self.error_string.replace(value.get().expect("The value needs to be of type `String`."));},
//"files" =>  {  self.files.replace(value.get_owned().expect("The value needs to be of type `Vec<File>`."));},
//"file-stats" => {   self.file_stats.replace(value.get().expect("The value needs to be of type `Vec<FileStats>`."));},
//"priorities" =>  {  self.priorities.replace(value.get().expect("The value needs to be of type `Vec<u8>`."));},
//"wanted" =>  {  self.wanted.replace(value.get().expect("The value needs to be of type `Vec<u8>`."));},
//"peers" =>  {  self.peers.replace(value.get().expect("The value needs to be of type `Vec<Peer>`."));},
//"trackers" =>  {  self.trackers.replace(value.get().expect("The value needs to be of type `Vec<Tracker>`."));},
//"tracker-stats" =>  {  self.tracker_stats.replace(value.get().expect("The value needs to be of type `Vec<TrackerStat>`."));},
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
"name" => self.name.borrow().to_value(),
"eta" => self.eta.get().to_value(),
"size-when-done" => self.size_when_done.get().to_value(),
"seeder-count" => self.seeder_count.get().to_value(),
"leecher-count" => self.leecher_count.get().to_value(),
"status" => self.status.get().to_value(),
"download-dir" => self.download_dir.borrow().to_value(),
"comment" => self.comment.borrow().to_value(),
"hash-string" => self.hash_string.borrow().to_value(),
"rate-download" => self.rate_download.get().to_value(),
"rate-upload" => self.rate_upload.get().to_value(),
"upload-ratio" => self.upload_ratio.get().to_value(),
"seed-ratio-limit" => self.seed_ratio_limit.get().to_value(),
"priority" => self.priority.get().to_value(),
"done-date" => self.done_date.get().to_value(),
"percent-complete" => self.percent_complete.get().to_value(),
"downloaded-ever" => self.downloaded_ever.get().to_value(),
"uploaded-ever" => self.uploaded_ever.get().to_value(),
"corrupt-ever" => self.corrupt_ever.get().to_value(),
//"labels" => self.labels.borrow().to_value(),
"piece-count" => self.piece_count.get().to_value(),
"pieces" => self.pieces.borrow().to_value(),
"error" => self.error.get().to_value(),
"error-string" => self.error_string.borrow().to_value(),
//"files" => self.files.borrow().to_value(),
//"file-stats" => self.file_stats.borrow().to_value(),
//"priorities" => self.priorities.borrow().to_value(),
//"wanted" => self.wanted.borrow().to_value(),
//"peers" => self.peers.borrow().to_value(),
//"trackers" => self.trackers.borrow().to_value(),
//"tracker-stats" => self.tracker_stats.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
