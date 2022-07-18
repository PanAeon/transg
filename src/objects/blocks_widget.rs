
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct BlocksWidget(ObjectSubclass<imp::BlocksWidget>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl BlocksWidget {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create `BlocksWidget`.")
    }
}

impl Default for BlocksWidget {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::Cell;

    use glib::{BindingFlags, ParamFlags, ParamSpec, ParamSpecInt, Value};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    // Object holding the state
    #[derive(Default)]
    pub struct BlocksWidget {
        number: Cell<i32>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for BlocksWidget {
        const NAME: &'static str = "BlocksWidget";
        type Type = super::BlocksWidget;
        type ParentType = gtk::DrawingArea;
    }

    // ANCHOR: object_impl
    // Trait shared by all GObjects
    impl ObjectImpl for BlocksWidget {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecInt::new(
                    // Name
                    "number",
                    // Nickname
                    "number",
                    // Short description
                    "number",
                    // Minimum value
                    i32::MIN,
                    // Maximum value
                    i32::MAX,
                    // Default value
                    0,
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "number" => {
                    let input_number = value.get().expect("The value needs to be of type `i32`.");
                    self.number.replace(input_number);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "number" => self.number.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Bind label to number
            // `SYNC_CREATE` ensures that the label will be immediately set
            obj.bind_property("number", obj, "label")
                .flags(BindingFlags::SYNC_CREATE)
                .build();
        }
    }
    // ANCHOR_END: object_impl

    // Trait shared by all widgets
    impl WidgetImpl for BlocksWidget {}

    // ANCHOR: button_impl
    // Trait shared by all buttons
    impl DrawingAreaImpl for BlocksWidget {

       //fn clicked(&self, button: &Self::Type) {
       //     let incremented_number = self.number.get() + 1;
       //     button.set_property("number", &incremented_number);
       // }
    }
    // ANCHOR_END: button_impl
}
