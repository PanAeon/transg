use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct CategoryObject(ObjectSubclass<imp::CategoryObject>);
}

impl CategoryObject {
    pub fn new(
        name: String,
        count: u64,
        status: i64,
        is_folder: bool,
        download_dir: String 
    ) -> Self {
        Object::new(&[
            ("name", &name),
            ("count", &count),
            ("status", &status),
            ("is-folder", &is_folder),
            ("download-dir", &download_dir)
        ])
        .expect("Failed to create 'CategoryObject'")
    }
}

mod imp {

    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean,ParamSpecInt64, ParamSpecUInt64, ParamSpecString, Value};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct CategoryObject {
        name: RefCell<String>,
        count: Cell<u64>,
        status: Cell<i64>,
        is_folder: Cell<bool>,
        download_dir: RefCell<String>
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for CategoryObject {
        const NAME: &'static str = "CategoryObject";
        type Type = super::CategoryObject;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for CategoryObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new( "name", "name", "name", None, ParamFlags::READWRITE,),
                    ParamSpecUInt64::new( "count", "count", "count", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecInt64::new("status", "status", "status", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecBoolean::new("is-folder", "is-folder", "is-folder", false, ParamFlags::READWRITE,),
                    ParamSpecString::new( "download-dir", "download-dir", "download-dir", None, ParamFlags::READWRITE,),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "name" => { self.name .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "count" => { self.count .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "status" => { self.status .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "is-folder" => { self.is_folder.replace(value.get().expect("The value needs to be of type `i32`.")); }
                "download-dir" => { self.download_dir.replace(value.get().expect("The value needs to be of type `i32`.")); }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => self.name.borrow().to_value(),
                "count" => self.count.get().to_value(),
                "status" => self.status.get().to_value(),
                "is-folder" => self.is_folder.get().to_value(),
                "download-dir" => self.download_dir.borrow().to_value(),
                other => unimplemented!("{}", other),
            }
        }
    }
}
