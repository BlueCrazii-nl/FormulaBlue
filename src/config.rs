use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::fs;
use std::process::exit;
use log::{info, debug};

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub f1_username:    String,
    pub f1_password:    String,
    pub rtmp_ingest:    String,
    pub ned_key:        String,
    pub eng_key:        String,
    pub data_key:       String,
    pub test_session_id: Option<String>,
    pub ffmpeg_command: String,
    pub data_command:   String,
}

impl Config {
    pub fn new() -> Result<Self> {
        let path = PathBuf::from("/etc/formulablue");
        if !path.exists() {
            debug!("Config dir does not exist, creating");
            fs::create_dir_all(&path)?;
        }

        let config_path = path.join("config.yml");
        if !config_path.exists() {
            debug!("Config file does not exist, creating default");
            let mut f = fs::File::create(&config_path)?;
            serde_yaml::to_writer(&mut f, &Self::default())?;
            info!("Created default configuration file at {:?}", &config_path);
            exit(0);
        }

        let f = fs::File::open(&config_path)?;
        Ok(serde_yaml::from_reader(&f)?)
    }
}