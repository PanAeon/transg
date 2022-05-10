
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct TrackerObject(ObjectSubclass<imp::TrackerObject>);
}

impl TrackerObject {
    pub fn new(
      id: &u64,
      announce: &String,
      tier: &u64,
      leecher_count: &i64,
      host: &String,
      scrape: &String,
      seeder_count: &i64,
      last_announce_peer_count: &u64,
      last_announce_result: &String,
      last_announce_time: &u64,
    ) -> Self {
        Object::new(&[
          ("id", &id),
          ("announce", &announce),
          ("scrape", &scrape),
          ("tier", &tier),
          ("leecher-count", &leecher_count),
          ("host", &host),
          ("seeder-count", &seeder_count),
          ("last-announce-peer-count", &last_announce_peer_count),
          ("last-announce-result", &last_announce_result),
          ("last-announce-time", &last_announce_time),
        ])
        .expect("Failed to create 'TrackerObject'")
    }
}

mod imp {

use gtk::{glib::ParamSpecString, subclass::prelude::*};
use gtk::glib;
use glib::{ParamFlags, ParamSpec, ParamSpecUInt64, Value, ParamSpecInt64};
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::{RefCell, Cell};

    #[derive(Default)]
    pub struct TrackerObject {
      id: Cell<u64>,
      announce: RefCell<String>,
      scrape: RefCell<String>,
      tier: Cell<u64>,
      leecher_count: Cell<i64>,
      host: RefCell<String>,
      seeder_count: Cell<i64>,
      last_announce_peer_count: Cell<u64>,
      last_announce_result: RefCell<String>,
      last_announce_time: Cell<u64>,
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
                  ParamSpecInt64::new( "leecher-count", "leecher-count", "leecher-count", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                  ParamSpecString::new( "host", "host", "host", None, ParamFlags::READWRITE,),
                  ParamSpecInt64::new( "seeder-count", "seeder-count", "seeder-count", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),
                  ParamSpecUInt64::new( "last-announce-peer-count", "last-announce-peer-count", "last-announce-peer-count", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
                  ParamSpecString::new( "last-announce-result", "last-announce-result", "last-announce-result", None, ParamFlags::READWRITE,),
                  ParamSpecUInt64::new( "last-announce-time", "last-announce-time", "last-announce-time", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),    
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
                "leecher-count" => {  self.leecher_count.replace(value.get().expect("The value needs to be of type `i64`."));},
"host" => {  self.host.replace(value.get().expect("The value needs to be of type `String`."));},
"seeder-count" => {  self.seeder_count.replace(value.get().expect("The value needs to be of type `i64`."));},
"last-announce-peer-count" => {  self.last_announce_peer_count.replace(value.get().expect("The value needs to be of type `u64`."));},
"last-announce-result" => {  self.last_announce_result.replace(value.get().expect("The value needs to be of type `String`."));},
"last-announce-time" => {  self.last_announce_time.replace(value.get().expect("The value needs to be of type `u64`."));},
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
"announce" => self.announce.borrow().to_value(),
"scrape" => self.scrape.borrow().to_value(),
"tier" => self.tier.get().to_value(),
                "leecher-count" => self.leecher_count.get().to_value(),
"host" => self.host.borrow().to_value(),
"seeder-count" => self.seeder_count.get().to_value(),
"last-announce-peer-count" => self.last_announce_peer_count.get().to_value(),
"last-announce-result" => self.last_announce_result.borrow().to_value(),
"last-announce-time" => self.last_announce_time.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
