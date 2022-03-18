use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use std::io::{Read, Write};
use std::process::exit;
use log::{info, debug};

const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_FOLDER: &str = "/etc/formulablue";

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct F1TV {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Streams {
    pub ned: bool,
    pub eng: bool,
    pub data: bool,
    pub stream_for: i64,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Commands {
    pub ned: Option<String>,
    pub eng: Option<String>,
    pub data: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub f1tv: F1TV,
    pub streams: Streams,
    pub commands: Commands,
}

impl Config {
    pub fn new() -> Result<Self> {
        let path = PathBuf::from(CONFIG_FOLDER);
        if !path.exists() {
            debug!("Config dir does not exist, creating");
            fs::create_dir_all(&path)?;
        }

        let config_path = path.join(CONFIG_FILE_NAME);
        if !config_path.exists() {
            debug!("Config file does not exist, creating default");
            let mut f = fs::File::create(&config_path)?;

            let toml = toml::to_string_pretty(&Config::default())?;
            f.write_all(toml.as_bytes())?;

            info!("Created default configuration file at {:?}", &config_path);
            exit(0);
        }

        let mut f = fs::File::open(&config_path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let config: Self = toml::from_slice(&buf)?;
        Ok(config)
    }
}