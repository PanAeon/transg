use crate::objects::TorrentDetailsObject;
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct TorrentDetailsGrid(ObjectSubclass<imp::TorrentDetailsGrid>)
        @extends gtk::Grid, gtk::Widget,
        @implements gtk::Native, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}
impl TorrentDetailsGrid {
    pub fn new(details: &TorrentDetailsObject) -> Self {
        Object::new(&[("details", details)]).expect("Failed to create TorrentDetailsGrid")
    }
}

mod imp {

    use crate::objects::TorrentDetailsObject;
    use crate::utils::format_time;
    use crate::utils::{format_download_speed, format_eta, format_size};
    use glib::subclass::InitializingObject;
    use glib::{ParamFlags, ParamSpec, ParamSpecObject, Value};
    use gtk::{glib, DrawingArea};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{CompositeTemplate, Label};
    use once_cell::sync::Lazy;
    use std::cell::RefCell;
    use glib::clone;
    use gtk::prelude::*;
    use gtk::glib::prelude::*;
    use bit_vec::BitVec;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/transgression/torrent_detail.ui")]
    pub struct TorrentDetailsGrid {
        #[template_child]
        pub name: TemplateChild<Label>,
        pub details: RefCell<TorrentDetailsObject>,
        #[template_child]
        pub hash: TemplateChild<Label>,
        #[template_child]
        pub comment: TemplateChild<Label>,
        #[template_child]
        pub location: TemplateChild<Label>,
        #[template_child]
        pub status: TemplateChild<Label>,
        #[template_child]
        pub leechers: TemplateChild<Label>,
        #[template_child]
        pub seeders: TemplateChild<Label>,
        #[template_child]
        pub eta: TemplateChild<Label>,
        #[template_child]
        pub size: TemplateChild<Label>,
        #[template_child]
        pub rate_down: TemplateChild<Label>,
        #[template_child]
        pub rate_up: TemplateChild<Label>,
        #[template_child]
        pub ratio: TemplateChild<Label>,
        #[template_child]
        pub ratio_limit: TemplateChild<Label>,
        #[template_child]
        pub priority: TemplateChild<Label>,
        #[template_child]
        pub completed: TemplateChild<Label>,
        #[template_child]
        pub downloaded: TemplateChild<Label>,
        #[template_child]
        pub uploaded: TemplateChild<Label>,
        #[template_child]
        pub corrupted: TemplateChild<Label>,
        #[template_child]
        pub completed_at: TemplateChild<Label>,
        #[template_child]
        pub error: TemplateChild<Label>,
        #[template_child]
        pub pieces: TemplateChild<DrawingArea>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TorrentDetailsGrid {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "TorrentDetailsGrid";
        type Type = super::TorrentDetailsGrid;
        type ParentType = gtk::Grid;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for TorrentDetailsGrid {
        fn constructed(&self, obj: &Self::Type) {
            // Call "constructed" on parent
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "details",
                    "details",
                    "details",
                    TorrentDetailsObject::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }
        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "details" => {
                    let details = value.get_owned::<TorrentDetailsObject>().expect("foo");
                    //let details = value.get::<TorrentDetailsObject>().expect("Expect torrent details");
                    details
                        .property_expression("name")
                        .bind(&self.name.get(), "label", gtk::Widget::NONE); // TODO: what to do on unbind?
                    details
                        .property_expression("done-date")
                        .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                            format_time(i)
                        }))
                        .bind(&self.completed_at.get(), "label", gtk::Widget::NONE);
                    details
                        .property_expression("corrupt-ever")
                        .bind(&self.corrupted.get(), "label", gtk::Widget::NONE);
                    details
                        .property_expression("uploaded-ever")
                        .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                            format_size(i.try_into().unwrap())
                        }))
                        .bind(&self.uploaded.get(), "label", gtk::Widget::NONE);
                    details
                        .property_expression("downloaded-ever")
                        .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                            format_size(i.try_into().unwrap())
                        }))
                        .bind(&self.downloaded.get(), "label", gtk::Widget::NONE);
                    details.property_expression("percent-complete").bind(
                        &self.completed.get(),
                        "label",
                        gtk::Widget::NONE,
                    );
                    details
                        .property_expression("priority")
                        .bind(&self.priority.get(), "label", gtk::Widget::NONE);
                    details.property_expression("seed-ratio-limit").bind(
                        &self.ratio_limit.get(),
                        "label",
                        gtk::Widget::NONE,
                    );
                    details
                        .property_expression("upload-ratio")
                        .bind(&self.ratio.get(), "label", gtk::Widget::NONE);
                    details
                        .property_expression("rate-upload")
                        .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                            format_download_speed(i.try_into().unwrap())
                        }))
                        .bind(&self.rate_down.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("size-when-done")
                        .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                            format_size(i.try_into().unwrap())
                        }))
                        .bind(&self.size.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("eta")
                        .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: i64| {
                            format_eta(i)
                        }))
                        .bind(&self.eta.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("seeder-count")
                        .bind(&self.seeders.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("leecher-count")
                        .bind(&self.leechers.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("status")
                        .bind(&self.status.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("download-dir")
                        .bind(&self.location.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("comment")
                        .bind(&self.comment.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("hash-string")
                        .bind(&self.hash.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("error-string")
                        .bind(&self.error.get(), "label", gtk::Widget::NONE);

                    details
                        .property_expression("rate-download")
                        .chain_closure::<String>(gtk::glib::closure!(|_: Option<gtk::glib::Object>, i: u64| {
                            format_download_speed(i.try_into().unwrap())
                        }))
                        .bind(&self.rate_down.get(), "label", gtk::Widget::NONE);
                    let pieces = self.pieces.get();
                    pieces.set_content_height(25);
                    pieces.set_width_request(1200);
                    //pieces.set_content_width(1200);
                    pieces.set_hexpand(true);
                    //pieces.set_content_width(1200);
                     
                    details
                        .property_expression("pieces")
                        .watch(gtk::Widget::NONE, clone!(@weak details, @weak pieces => move || {
                            let n = details.property_value("piece-count").get::<u64>().expect("fjkds");
                            let s = details.property_value("pieces").get::<String>().expect("fklaksd");
                            let percent_done = details.property_value("percent-complete").get::<f64>().expect("zckjf");
                            let bytes = base64::decode(&s).expect("wrong bitfield?");
                            let mut bv = BitVec::from_bytes(&bytes);
                            bv.truncate(n as usize);
                    pieces.set_draw_func(move |_, ctx, width, height| {
                        if n > 0 {
                      let piece_width = width as f64 / n as f64;
                      //////println!("piece width: {}, count: {}, ", piece_width, n);
                      ctx.set_source_rgb(1.0, 1.0, 1.0); 
                      ctx.paint().expect("no paint for you");
                      ctx.set_source_rgb (0.082, 0.325, 0.62);
                      ctx.rectangle(0.0, 0.0, percent_done * width as f64, 6.0);
                      ctx.fill().expect("no fill for you");
                      ctx.set_source_rgb (0.11, 0.11, 0.11);
                      ctx.rectangle(0.0, 6.0,  width as f64, 6.0);
                      ctx.fill().expect("no fill for you");
                      ctx.set_source_rgb (0.082, 0.325, 0.62);
                      for (i, x) in bv.iter().enumerate() {
                          if x {
                              ctx.rectangle(i as f64 * piece_width, 12.0, piece_width + 0.5, height as f64 - 12.0);
                              ctx.fill().expect("no fill for you");
                          }
                      }
                        }
                    });
                        }));

                    self.details.replace(details);
                }
                _ => unimplemented!(),
            }
        }
        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "details" => self.details.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    // Trait shared by all widgets
    impl WidgetImpl for TorrentDetailsGrid {}

    impl GridImpl for TorrentDetailsGrid {}
}
