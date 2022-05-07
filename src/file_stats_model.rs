
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct FileStatsObject(ObjectSubclass<imp::FileStatsObject>);
}

impl FileStatsObject {
    pub fn new(
      wanted: &bool,
priority: &u64,
bytes_completed: &u64,
    ) -> Self {
        Object::new(&[
          ("wanted", &wanted),
("priority", &priority),
("bytes-completed", &bytes_completed),
        ])
        .expect("Failed to create 'FileStatsObject'")
    }
}

mod imp {

use gtk::glib::{self, ParamSpecBoolean};
use glib::{ParamFlags, ParamSpec, ParamSpecUInt64, Value};
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::Cell;
use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct FileStatsObject {
      wanted: Cell<bool>,
priority: Cell<u64>,
bytes_completed: Cell<u64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for FileStatsObject {
        const NAME: &'static str = "FileStatsObject";
        type Type = super::FileStatsObject;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for FileStatsObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                  ParamSpecBoolean::new( "wanted", "wanted", "wanted", false, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "priority", "priority", "priority", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "bytes-completed", "bytes-completed", "bytes-completed", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),    
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "wanted" =>  { self.wanted.replace(value.get().expect("The value needs to be of type `bool`."));},
"priority" => { self.priority.replace(value.get().expect("The value needs to be of type `u64`."));},
"bytes-completed" => { self.bytes_completed.replace(value.get().expect("The value needs to be of type `u64`."));},
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "wanted" => self.wanted.get().to_value(),
"priority" => self.priority.get().to_value(),
"bytes-completed" => self.bytes_completed.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
