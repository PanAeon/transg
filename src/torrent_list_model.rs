use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct TorrentInfo(ObjectSubclass<imp::TorrentInfo>);
}

impl TorrentInfo {
    pub fn new(id: i64, name: String, status: i64, percent_complete: f64, rate_upload: i64, total_size: i64, download_dir: String, added_date: i64) -> Self {
        Object::new(&[
            ("id", &id),
            ("name", &name),
            ("status", &status),
            ("percent-complete", &percent_complete),
            ("rate-upload", &rate_upload),
            ("total-size", &total_size),
            ("download-dir", &download_dir),
            ("added-date", &added_date),
        ]).expect("Failed to create 'TorrentInfo'")
    }
}

mod imp {

    use glib::{ParamFlags, ParamSpec, ParamSpecString, ParamSpecDouble, ParamSpecInt64, Value};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use std::cell::{RefCell, Cell};

    #[derive(Default)]
    pub struct TorrentInfo {
        id: Cell<i64>,
        name: RefCell<String>,
        status: Cell<i64>,
        percent_complete: Cell<f64>,
        rate_upload: Cell<i64>,
        total_size: Cell<i64>,
        download_dir: RefCell<String>,
        added_date: Cell<i64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for TorrentInfo {
        const NAME: &'static str = "TorrentInfo";
        type Type = super::TorrentInfo;
    }

    // Trait shared by all GObjects
    impl ObjectImpl for TorrentInfo {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecInt64::new( "id", "id", "id", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE),
                    ParamSpecString::new( "name", "name", "name", None, ParamFlags::READWRITE),
                    ParamSpecInt64::new( "status", "status", "status", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE),
                    ParamSpecDouble::new( "percent-complete", "percent-complete", "percent-complete", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE),
                    ParamSpecInt64::new( "rate-upload", "rate-upload", "rate-upload", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE),
                    ParamSpecInt64::new( "total-size", "total-size", "total-size", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE),
                    ParamSpecString::new( "download-dir", "download-dir", "download-dir", None, ParamFlags::READWRITE),
                    ParamSpecInt64::new( "added-date", "added-date", "added-date", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" => { self.id.replace(value.get().expect("The value needs to be of type `i32`.")); },
                "name" => { self.name.replace(value.get().expect("The value needs to be of type `i32`.")); },
                "status" => { self.status.replace(value.get().expect("The value needs to be of type `i32`.")); },
                "percent-complete" => { self.percent_complete.replace(value.get().expect("The value needs to be of type `i32`.")); },
                "rate-upload" => { self.rate_upload.replace(value.get().expect("The value needs to be of type `i32`.")); },
                "total-size" => { self.total_size.replace(value.get().expect("The value needs to be of type `i32`.")); },
                "download-dir" => { self.download_dir.replace(value.get().expect("The value needs to be of type `i32`.")); },
                "added-date" => { self.added_date.replace(value.get().expect("The value needs to be of type `i32`.")); },
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "name" => self.name.borrow().to_value(),
                "status" => self.status.get().to_value(),
                "percent-complete" => self.percent_complete.get().to_value(),
                "rate-upload" => self.rate_upload.get().to_value(),
                "total-size" => self.total_size.get().to_value(),
                "download-dir" => self.download_dir.borrow().to_value(),
                "added-date" => self.added_date.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
