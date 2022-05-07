use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct TorrentInfo(ObjectSubclass<imp::TorrentInfo>);
}

impl TorrentInfo {
    pub fn new(
        id: i64,
        name: String,
        status: i64,
        percent_done: f64,
        error: i64,
        error_string: String,
        eta: i64,
        queue_position: i64,
        is_finished: bool,
        is_stalled: bool,
        metadata_percent_complete: f64,
        peers_connected: i64,
        rate_download: i64,
        rate_upload: i64,
        recheck_progress: f64,
        size_when_done: i64,
        download_dir: String,
        uploaded_ever: i64,
        upload_ratio: f64,
        added_date: i64,
    ) -> Self {
        Object::new(&[
            ("id", &id),
            ("name", &name),
            ("status", &status),
            ("percent-done", &percent_done),
            ("error", &error),
            ("error-string", &error_string),
            ("eta", &eta),
            ("queue-position", &queue_position),
            ("is-finished", &is_finished),
            ("is-stalled", &is_stalled),
            ("metadata-percent-complete", &metadata_percent_complete),
            ("peers-connected", &peers_connected),
            ("rate-download", &rate_download),
            ("rate-upload", &rate_upload),
            ("recheck-progress", &recheck_progress),
            ("size-when-done", &size_when_done),
            ("download-dir", &download_dir),
            ("uploaded-ever", &uploaded_ever),
            ("upload-ratio", &upload_ratio),
            ("added-date", &added_date),
        ])
        .expect("Failed to create 'TorrentInfo'")
    }
}

mod imp {

    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecInt64, ParamSpecString, Value};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct TorrentInfo {
        id: Cell<i64>,
        name: RefCell<String>,
        status: Cell<i64>,
        percent_done: Cell<f64>,
        error: Cell<i64>,
        error_string: RefCell<String>,
        eta: Cell<i64>,
        queue_position: Cell<i64>,
        is_finished: Cell<bool>,
        is_stalled: Cell<bool>,
        metadata_percent_complete: Cell<f64>,
        peers_connected: Cell<i64>,
        rate_download: Cell<i64>,
        rate_upload: Cell<i64>,
        recheck_progress: Cell<f64>,
        size_when_done: Cell<i64>,
        download_dir: RefCell<String>,
        uploaded_ever: Cell<i64>,
        upload_ratio: Cell<f64>,
        added_date: Cell<i64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for TorrentInfo {
        const NAME: &'static str = "TorrentInfo";
        type Type = super::TorrentInfo;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for TorrentInfo {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecInt64::new( "id", "id", "id", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecString::new("name", "name", "name", None, ParamFlags::READWRITE),
                    ParamSpecInt64::new( "status", "status", "status", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecDouble::new( "percent-done", "percent-done", "percent-done", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "error", "error", "error", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecString::new( "error-string", "error-string", "error-string", None, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "eta", "eta", "eta", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "queue-position", "queue-position", "queue-position", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecBoolean::new("is-finished", "is-finished", "is-finished", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("is-stalled", "is-stalled", "is-stalled", false, ParamFlags::READWRITE),
                    ParamSpecDouble::new( "metadata-percent-complete", "metadata-percent-complete", "metadata-percent-complete", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "peers-connected", "peers-connected", "peers-connected", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "rate-download", "rate-download", "rate-download", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "rate-upload", "rate-upload", "rate-upload", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecDouble::new( "recheck-progress", "recheck-progress", "recheck-progress", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "size-when-done", "size-when-done", "size-when-done", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecString::new( "download-dir", "download-dir", "download-dir", None, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "uploaded-ever", "uploaded-ever", "uploaded-ever", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecDouble::new( "upload-ratio", "upload-ratio", "upload-ratio", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),
                    ParamSpecInt64::new( "added-date", "added-date", "added-date", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" => { self.id .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "name" => { self.name .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "status" => { self.status .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "percent-done" => { self.percent_done.replace(value.get().expect("The value needs to be of type `i32`.")); }
                "error" => { self.error .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "error-string" => { self.error_string .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "eta" => { self.eta .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "queue-position" => { self.queue_position .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "is-finished" => { self.is_finished .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "is-stalled" => { self.is_stalled .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "metadata-percent-complete" => { self.metadata_percent_complete.replace(value.get().expect("The value needs to be of type `i32`.")); }
                "peers-connected" => { self.peers_connected .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "rate-download" => { self.rate_download .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "rate-upload" => { self.rate_upload .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "recheck-progress" => { self.recheck_progress .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "size-when-done" => { self.size_when_done .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "download-dir" => { self.download_dir .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "uploaded-ever" => { self.uploaded_ever .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "upload-ratio" => { self.upload_ratio .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "added-date" => { self.added_date .replace(value.get().expect("The value needs to be of type `i32`.")); }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "name" => self.name.borrow().to_value(),
                "status" => self.status.get().to_value(),
                "percent-done" => self.percent_done.get().to_value(),
                "error" => self.error.get().to_value(),
                "error-string" => self.error_string.borrow().to_value(),
                "eta" => self.eta.get().to_value(),
                "queue-position" => self.queue_position.get().to_value(),
                "is-finished" => self.is_finished.get().to_value(),
                "is-stalled" => self.is_stalled.get().to_value(),
                "metadata-percent-complete" => self.metadata_percent_complete.get().to_value(),
                "peers-connected" => self.peers_connected.get().to_value(),
                "rate-download" => self.rate_download.get().to_value(),
                "rate-upload" => self.rate_upload.get().to_value(),
                "recheck-progress" => self.recheck_progress.get().to_value(),
                "size-when-done" => self.size_when_done.get().to_value(),
                "download-dir" => self.download_dir.borrow().to_value(),
                "uploaded-ever" => self.uploaded_ever.get().to_value(),
                "upload-ratio" => self.upload_ratio.get().to_value(),
                "added-date" => self.added_date.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
