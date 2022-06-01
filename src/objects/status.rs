use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct StatusInfo(ObjectSubclass<imp::StatusInfo>);
}

impl StatusInfo {
    pub fn new(id: i64, count: u64) -> Self {
        Object::new(&[("id", &id), ("count", &count)]).expect("Failed to create 'StatusInfo'")
    }
}

mod imp {

    use glib::{ParamFlags, ParamSpec, ParamSpecInt64, ParamSpecUInt64, Value};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use std::cell::Cell;

    #[derive(Default)]
    pub struct StatusInfo {
        id: Cell<i64>,
        count: Cell<u64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for StatusInfo {
        const NAME: &'static str = "StatusInfo";
        type Type = super::StatusInfo;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for StatusInfo {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecInt64::new( "id", "id", "id", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecUInt64::new( "count", "count", "count", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" => { self.id .replace(value.get().expect("The value needs to be of type `i32`.")); }
                "count" => { self.count .replace(value.get().expect("The value needs to be of type `i32`.")); }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "count" => self.count.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
