
// now our time haz come, my fellow brothers!
// should have written that fucking macro..
fn main() {
    let class_name = "PeerObject";
    // rust name, rust type
    let fields = vec![
  ("address", "String"),
  ("client_name", "String"),
  ("progress", "f64"),
  ("rate_to_client", "u64"),
  ("rate_to_peer", "u64"),
  ("flag_str", "String"),
    ];


    let constructor_params = fields.iter().map(|f| {
      let (name, _t) = f; 
      format!("{}: &{},", name, _t)
    }).collect::<Vec<String>>().join("\n");

    let constructor_tuples = fields.iter().map(|f| {
      let (name, _) = f; 
      format!("(\"{}\", &{}),", name.replace("_", "-"), name)
    }).collect::<Vec<String>>().join("\n");

    let struct_fields = fields.iter().map(|(name, type_)| {
      let type_ = match *type_ {
        x@("u64" | "u8" | "f64" | "i8" | "bool" | "i64") => format!("Cell<{}>", x), 
        other  => format!("RefCell<{}>", other)
      };
      format!("{}: {},", name, type_)
    }).collect::<Vec<String>>().join("\n");

    let properties = fields.iter().map(|(name, type_)| {
      let name = name.replace("_", "-");
      match *type_ {
        "u64" => format!("ParamSpecUInt64::new( \"{name}\", \"{name}\", \"{name}\", u64::MIN, u64::MAX, 0, ParamFlags::READWRITE,),"),
        "f64" => format!("ParamSpec::new_double( \"{name}\", \"{name}\", \"{name}\", f64::MIN, f64::MAX, 0.0, ParamFlags::READWRITE,),"),
        "i8"  => format!("ParamSpec::new_char( \"{name}\", \"{name}\", \"{name}\", i8::MIN, i8::MAX, 0, ParamFlags::READWRITE,),"),
        "u8"  => format!("ParamSpec::new_uchar( \"{name}\", \"{name}\", \"{name}\", u8::MIN, u8::MAX, 0, ParamFlags::READWRITE,),"),
        "bool" => format!("ParamSpec::new_boolean( \"{name}\", \"{name}\", \"{name}\", false, ParamFlags::READWRITE,),"),
        "i64" => format!("ParamSpecInt64::new( \"{name}\", \"{name}\", \"{name}\", i64::MIN, i64::MAX, 0, ParamFlags::READWRITE,),"),
        "String" => format!("ParamSpec::new_string( \"{name}\", \"{name}\", \"{name}\", None, ParamFlags::READWRITE,),"),
        other if other.starts_with("Vec") => format!("ParamSpec::new_value_array(\"{name}\", \"{name}\", \"{name}\", param_spec_of_children, ParamFlags::READWRITE,),"),
        other  => format!("ParamSpec::new_object( \"{name}\", \"{name}\", \"{name}\", {other}::static_type(), ParamFlags::READWRITE,),"),
      }
    }).collect::<Vec<String>>().join("\n");
    
    let set_property = fields.iter().map(|(name, type_)| {
      let gname = name.replace("_", "-");
      format!("\"{gname}\" =>  self.{name}.replace(value.get().expect(\"The value needs to be of type `{type_}`.\")),") 
    }).collect::<Vec<String>>().join("\n");

                

    let get_property = fields.iter().map(|(name, type_)| {
      let gname = name.replace("_", "-");
      match *type_ {
        "u64" | "u8" | "f64" | "i8" | "bool" | "i64" => format!("\"{gname}\" => self.{name}.get().to_value(),"), 
        _  => format!("\"{gname}\" => self.{name}.borrow().to_value(),"), 
      }
    }).collect::<Vec<String>>().join("\n");

    let res = format!(r#"
use glib::Object;
use gtk::glib;
use glib::{{ParamFlags, ParamSpec, ParamSpecInt64, ParamSpecUInt64, Value}};
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::Cell;
use gtk::subclass::prelude::*;

glib::wrapper! {{
    pub struct {class_name}(ObjectSubclass<imp::{class_name}>);
}}

impl {class_name} {{
    pub fn new(
      {constructor_params}
    ) -> Self {{
        Object::new(&[
          {constructor_tuples}
        ])
        .expect("Failed to create '{class_name}'")
    }}
}}

mod imp {{

    #[derive(Default)]
    pub struct {class_name} {{
      {struct_fields}
    }}

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for {class_name} {{
        const NAME: &'static str = "{class_name}";
        type Type = super::{class_name};
    }}

    // Trait shared by all GObjects
    #[rustfmt::skip]
    impl ObjectImpl for {class_name} {{
        fn properties() -> &'static [ParamSpec] {{
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {{
                vec![
                  {properties}    
                ]
            }});
            PROPERTIES.as_ref()
        }}

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {{
            match pspec.name() {{
                {set_property}
                _ => unimplemented!(),
            }}
        }}

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {{
            match pspec.name() {{
                {get_property}
                _ => unimplemented!(),
            }}
        }}
    }}
}}"#);
  println!("{}", res);
}
/* 
   TorrentDetailsObject

    let fields = vec![
    ( "id", "u64"),
    ( "name", "String"),
    ( "eta", "i64"),
    ( "size_when_done", "u64"),
    ( "seeder_count", "i64"),
    ( "leecher_count", "i64"),
    ( "status", "u64"),
    ( "download_dir", "String"),
    ( "comment", "String"),
    ( "hash_string", "String"),
    ( "rate_download", "u64"),
    ( "rate_upload", "u64"),
    ( "upload_ratio", "f64"),
    ( "seed_ratio_limit", "u64"),
    ( "priority", "u64"),
    ( "done_date", "u64"),
    ( "percent_complete", "f64"),
    ( "downloaded_ever", "u64"),
    ( "uploaded_ever", "u64"),
    ( "corrupt_ever", "u64"),
    ( "labels", "Vec<String>"),
    ( "piece_count", "u64"),
    ( "pieces", "String"), // base64 encoded bitstring
    ( "files", "Vec<File>"),
    ( "file_stats", "Vec<FileStats>"),
    ( "priorities", "Vec<u8>"),
    ( "wanted", "Vec<u8>"),
    ( "peers", "Vec<Peer>"),
    ( "trackers", "Vec<Tracker>"),
    ( "tracker_stats", "Vec<TrackerStat>"),
    ];

    let class_name = "TrackerObject";
    // rust name, rust type
    let fields = vec![
 ("id", "u64"),
 ("announce", "String"),
 ("scrape", "String"),
 ("tier", "u64"),

    let class_name = "TrackerStatsObject";
    // rust name, rust type
    let fields = vec![
  ("leecher_count", "i64"),
  ("id", "u64"),
  ("host", "String"),
  ("scrape", "String"),
  ("seeder_count", "i64"),
  ("last_announce_peer_count", "u64"),
  ("last_announce_result", "String"),
  ("last_announce_time", "u64"),
    ];

    let class_name = "FileStatsObject";
    // rust name, rust type
    let fields = vec![
  ("wanted", "bool"),
  ("priority", "u64"),
  ("bytes_completed", "u64"),
    ];
    let class_name = "FileObject";
    // rust name, rust type
    let fields = vec![
    ("name", "String"),
    ("length", "u64"),
    */
