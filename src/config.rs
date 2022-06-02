use home::home_dir;
use serde::Deserialize;
use std::fs::{create_dir_all, write, File};
use std::io::BufReader;

#[derive(Deserialize, Debug, Clone)]
pub struct DirMapping {
    pub label: String,
    pub remote_path: String,
    pub local_path: Option<String>,
}
// hmm, now it's public mutable.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub connection_string: String,
    pub directories: Vec<DirMapping>,
    pub remote_base_dir: String,
    pub local_base_dir: String
}

const EMPTY_CONFIG: &str = r#"{
  "connection_string": "",
  "directories": []
}"#;

pub fn get_or_create_config() -> Config {
    let home = home_dir().expect("can't obtain user home directory");
    let config_dir = home.join(".config").join("transg");
    if !config_dir.exists() {
        create_dir_all(&config_dir).expect("can't create ~/.config/transg");
    }
    let config_path = config_dir.join("config.json");
    if !config_path.exists() {
        write(&config_path, EMPTY_CONFIG).expect(format!("Failed to create {:?}", &config_path).as_str());
    }
    let f = File::open(config_path).expect("can't open config file");
    let buff = BufReader::new(f);
    let config: Config = serde_json::from_reader(buff).expect("can't parse json config");
    config
}
