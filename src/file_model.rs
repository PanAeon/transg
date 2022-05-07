
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct FileObject(ObjectSubclass<imp::FileObject>);
}

impl FileObject {
    pub fn new(
      name: &String,
length: &u64,
bytes_completed: &u64,
    ) -> Self {
        Object::new(&[
          ("name", &name),
("length", &length),
("bytes-completed", &bytes_completed),
        ])
        .expect("Failed to create 'FileObject'")
    }
}

mod imp {

use gtk::glib::ParamSpecString;
use once_cell::sync::Lazy;
use std::cell::{RefCell, Cell};
use glib::{ParamFlags, ParamSpec, ParamSpecUInt64, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct FileObject {
      name: RefCell<String>,
length: Cell<u64>,
bytes_completed: Cell<u64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for FileObject {
        const NAME: &'static str = "FileObject";
        type Type = super::FileObject;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for FileObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                  ParamSpecString::new( "name", "name", "name", None, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "length", "length", "length", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "bytes-completed", "bytes-completed", "bytes-completed", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),    
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "name" => { self.name.replace(value.get().expect("The value needs to be of type `String`."));},
"length" => { self.length.replace(value.get().expect("The value needs to be of type `u64`."));},
"bytes-completed" => { self.bytes_completed.replace(value.get().expect("The value needs to be of type `u64`."));},
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => self.name.borrow().to_value(),
"length" => self.length.get().to_value(),
"bytes-completed" => self.bytes_completed.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
