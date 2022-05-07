
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct TrackerObject(ObjectSubclass<imp::TrackerObject>);
}

impl TrackerObject {
    pub fn new(
      id: &u64,
announce: &String,
scrape: &String,
tier: &u64,
    ) -> Self {
        Object::new(&[
          ("id", &id),
("announce", &announce),
("scrape", &scrape),
("tier", &tier),
        ])
        .expect("Failed to create 'TrackerObject'")
    }
}

mod imp {

use gtk::{glib::ParamSpecString, subclass::prelude::*};
use gtk::glib;
use glib::{ParamFlags, ParamSpec, ParamSpecUInt64, Value};
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::{RefCell, Cell};

    #[derive(Default)]
    pub struct TrackerObject {
      id: Cell<u64>,
announce: RefCell<String>,
scrape: RefCell<String>,
tier: Cell<u64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for TrackerObject {
        const NAME: &'static str = "TrackerObject";
        type Type = super::TrackerObject;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for TrackerObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                  ParamSpecUInt64::new( "id", "id", "id", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecString::new( "announce", "announce", "announce", None, ParamFlags::READWRITE,),
ParamSpecString::new( "scrape", "scrape", "scrape", None, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "tier", "tier", "tier", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),    
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" => {  self.id.replace(value.get().expect("The value needs to be of type `u64`."));},
"announce" => {  self.announce.replace(value.get().expect("The value needs to be of type `String`."));},
"scrape" => {  self.scrape.replace(value.get().expect("The value needs to be of type `String`."));},
"tier" => {  self.tier.replace(value.get().expect("The value needs to be of type `u64`."));},
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
"announce" => self.announce.borrow().to_value(),
"scrape" => self.scrape.borrow().to_value(),
"tier" => self.tier.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
