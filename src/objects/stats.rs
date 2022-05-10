
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct Stats(ObjectSubclass<imp::Stats>);
}

impl Stats {
    pub fn new(
        upload: u64,
        download: u64,
        free_space: u64,
    ) -> Self {
        Object::new(&[
            ("upload", &upload),
            ("download", &download),
            ("free-space", &free_space)
        ])
        .expect("Failed to create 'Stats'")
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
    pub struct Stats {
        upload: Cell<u64>,
        download: Cell<u64>,
        free_space: Cell<u64>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Stats {
        const NAME: &'static str = "Stats";
        type Type = super::Stats;
    }

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for Stats {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecUInt64::new( "upload", "upload", "upload", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecUInt64::new( "download", "download", "download", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
                    ParamSpecUInt64::new( "free-space", "free-space", "free-space", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "upload" => { self.upload.replace(value.get().expect("The value needs to be of type `i32`.")); }
                "download" => { self.download.replace(value.get().expect("The value needs to be of type `i32`.")); }
                "free-space" => { self.free_space.replace(value.get().expect("The value needs to be of type `i32`.")); }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "upload" => self.upload.get().to_value(),
                "download" => self.download.get().to_value(),
                "free-space" => self.free_space.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
