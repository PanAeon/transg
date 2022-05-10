use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct FolderInfo(ObjectSubclass<imp::FolderInfo>);
}

impl FolderInfo {
    pub fn new(
        name: String,
        count: u64,
    ) -> Self {
        Object::new(&[
            ("name", &name),
            ("count", &count),
        ])
        .expect("Failed to create 'FolderInfo'")
    }
}

mod imp {

    use glib::{ParamFlags, ParamSpec, ParamSpecUInt64, ParamSpecString, Value};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct FolderInfo {
        name: RefCell<String>,
        count: Cell<u64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for FolderInfo {
        const NAME: &'static str = "FolderInfo";
        type Type = super::FolderInfo;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for FolderInfo {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new( "name", "name", "name", None, ParamFlags::READWRITE,),
                    ParamSpecUInt64::new( "count", "count", "count", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "name" => { self.name .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "count" => { self.count .replace(value.get().expect("The value needs to be of type `i32`.")); }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => self.name.borrow().to_value(),
                "count" => self.count.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
