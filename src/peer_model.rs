
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct PeerObject(ObjectSubclass<imp::PeerObject>);
}

impl PeerObject {
    pub fn new(
      address: &String,
client_name: &String,
progress: &f64,
rate_to_client: &u64,
rate_to_peer: &u64,
flag_str: &String,
    ) -> Self {
        Object::new(&[
          ("address", &address),
("client-name", &client_name),
("progress", &progress),
("rate-to-client", &rate_to_client),
("rate-to-peer", &rate_to_peer),
("flag-str", &flag_str),
        ])
        .expect("Failed to create 'PeerObject'")
    }
}

mod imp {

use gtk::glib::{self, ParamSpecString, ParamSpecDouble};
use glib::{ParamFlags, ParamSpec, ParamSpecUInt64, Value};
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::{RefCell, Cell};
use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct PeerObject {
      address: RefCell<String>,
client_name: RefCell<String>,
progress: Cell<f64>,
rate_to_client: Cell<u64>,
rate_to_peer: Cell<u64>,
flag_str: RefCell<String>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PeerObject {
        const NAME: &'static str = "PeerObject";
        type Type = super::PeerObject;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for PeerObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                  ParamSpecString::new( "address", "address", "address", None, ParamFlags::READWRITE,),
ParamSpecString::new( "client-name", "client-name", "client-name", None, ParamFlags::READWRITE,),
ParamSpecDouble::new( "progress", "progress", "progress", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "rate-to-client", "rate-to-client", "rate-to-client", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecUInt64::new( "rate-to-peer", "rate-to-peer", "rate-to-peer", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
ParamSpecString::new( "flag-str", "flag-str", "flag-str", None, ParamFlags::READWRITE,),    
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "address" => {  self.address.replace(value.get().expect("The value needs to be of type `String`."));},
"client-name" => {  self.client_name.replace(value.get().expect("The value needs to be of type `String`."));},
"progress" => {  self.progress.replace(value.get().expect("The value needs to be of type `f64`."));},
"rate-to-client" => {  self.rate_to_client.replace(value.get().expect("The value needs to be of type `u64`."));},
"rate-to-peer" => {  self.rate_to_peer.replace(value.get().expect("The value needs to be of type `u64`."));},
"flag-str" => {  self.flag_str.replace(value.get().expect("The value needs to be of type `String`."));},
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "address" => self.address.borrow().to_value(),
"client-name" => self.client_name.borrow().to_value(),
"progress" => self.progress.get().to_value(),
"rate-to-client" => self.rate_to_client.get().to_value(),
"rate-to-peer" => self.rate_to_peer.get().to_value(),
"flag-str" => self.flag_str.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
