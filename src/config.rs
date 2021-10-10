use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub f1_username:    Option<String>,
    pub f1_password:    Option<String>,
    pub rtmp_ingest:   Option<String>,
    pub ned_key:        Option<String>,
    pub eng_key:        Option<String>,
    pub data_key        Option<String>,
    pub tmpurl:         Option<String>,
    pub ffmpeg_command: Option<String>
}

impl Default for Config {
    fn default() -> Config {
        Config {
            f1_username: None,
            f1_password: None,
            rtmp_ingest: None,
            ned_key: None,
            eng_key: None,
            data_key: None
            tmpurl: None,
            ffmpeg_command: Some("ffmpeg -re -i \"{source_url}\" -c:a aac -ac 2 -b:a 128k -c:v libx264 -pix_fmt yuv420p -b:v 8000k -bufsize 3000k -s 1920x1080 -filter:v fps=50 -map 0:v:5 -map 0:m:language:{lang} -s 1920x1080 -f flv rtmp://{ingest}/{key}".to_string())
        }
    }
}

impl Config {
    pub fn verify(&self) -> (bool, &str) {
        if self.f1_username.is_none() {
            return (false, "f1_username");
        }

        if self.f1_password.is_none() {
            return (false, "f1_password");
        }

        if self.rtmp_ingest.is_none() {
            return (false, "rtmp_ingest");
        }

        if self.ned_key.is_none() {
            return (false, "ned_key");
        }

        if self.eng_key.is_none() {
            return (false, "eng_key");
        }

        if self.data_key.is_none() {
            return (false, "data_key");
        }

        if self.ffmpeg_command.is_none() {
            return (false, "ffmpeg_command");
        }

        (true, "")
    }
}

pub fn read() -> Config {
    #[cfg(windows)]
    let cfg_folder = {
        PathBuf::from(r#"C:\Program Files\FormulaBlue\"#)
    };

    #[cfg(unix)]
    let cfg_folder = {
        PathBuf::from(r#"/etc/formulablue/"#)
    };

    if !cfg_folder.exists() {
        std::fs::create_dir(cfg_folder.as_path()).expect("An issue occurred while creating the configuration folder.");
    }

    let mut cfg_file = cfg_folder.clone();
    cfg_file.push("config.yml");

    if !cfg_file.exists() {
        let cfg = serde_yaml::to_string(&Config::default()).expect("An issue occurred while serializing the default configuration.");
        std::fs::write(cfg_file.as_path(), cfg.as_bytes()).expect("An issue occurred while writing the default configuration to disk.");

        println!("No configuration file found. A new one has been created at {}. Please configure FormulaBlue and restart the application afterwards.", cfg_file.as_path().to_str().unwrap());
        std::process::exit(0);
    }

    let cfg_content = std::fs::read_to_string(cfg_file.as_path()).expect("An issue occurred while reading the configuration file.");
    let cfg = serde_yaml::from_str(&cfg_content).expect("An issue occurred while deserializing the configuration file.");

    cfg
}
