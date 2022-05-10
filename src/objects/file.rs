
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct FileObject(ObjectSubclass<imp::FileObject>);
}

impl FileObject {
    pub fn new(
      path: &String,
      size: &u64,
      progress: &f64,
      download: bool,
      priority: u8,
    ) -> Self {
        Object::new(&[
          ("path", &path),
          ("size", &size),
          ("progress", &progress),
          ("download", &download),
          ("priority", &priority)
        ])
        .expect("Failed to create 'FileObject'")
    }
}

mod imp {

use gtk::glib::ParamSpecBoolean;
use gtk::glib::ParamSpecString;
use gtk::glib::ParamSpecUChar;
use once_cell::sync::Lazy;
use std::cell::{RefCell, Cell};
use glib::{ParamFlags, ParamSpec, ParamSpecUInt64, Value, ParamSpecDouble};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct FileObject {
      path: RefCell<String>,
      size: Cell<u64>,
      progress: Cell<f64>,
      download: Cell<bool>,
      priority: Cell<u8>,
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
                  ParamSpecString::new( "path", "path", "path", None, ParamFlags::READWRITE,),
                  ParamSpecUInt64::new( "size", "size", "size", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
                  ParamSpecDouble::new( "progress", "progress", "progress", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),    
                  ParamSpecBoolean::new( "download", "download", "download", false, ParamFlags::READWRITE,),    
                  ParamSpecUChar::new( "priority", "priority", "priority", u8::MIN, u8::MAX, 0, ParamFlags::READWRITE,),    
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "path" => { self.path.replace(value.get().expect("The value needs to be of type `String`."));},
                "size" => { self.size.replace(value.get().expect("The value needs to be of type `u64`."));},
                "progress" => { self.progress.replace(value.get().expect("The value needs to be of type `u64`."));},
                "download" => { self.download.replace(value.get().expect("The value needs to be of type `u64`."));},
                "priority" => { self.priority.replace(value.get().expect("The value needs to be of type `u64`."));},
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "path" => self.path.borrow().to_value(),
                "size" => self.size.get().to_value(),
                "progress" => self.progress.get().to_value(),
                "download" => self.download.get().to_value(),
                "priority" => self.priority.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}