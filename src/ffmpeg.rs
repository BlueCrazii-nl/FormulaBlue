use std::process::Command;
use crate::config::Config;
use std::time::Duration;

pub fn stream(url: String, duration: i64, cfg: Config) {
    let ffmpeg_str = cfg.ffmpeg_command.unwrap();
    let ingest_url = cfg.rtmp_ingest.unwrap();

    let ffmpeg_str_ned = ffmpeg_str
        .replace("{source_url}", &url)
        .replace("{ingest}", &ingest_url)
        .replace("{lang}", "nld")
        .replace("{key}", &cfg.ned_key.unwrap());

    let ffmpeg_str_eng = ffmpeg_str
        .replace("{source_url}", &url)
        .replace("{ingest}", &ingest_url)
        .replace("{lang}", "eng")
        .replace("{key}", &cfg.eng_key.unwrap());

    //NED Stream
    std::thread::spawn(move || {
        let mut child = {
            Command::new("sh")
                .arg("-c")
                .arg(&ffmpeg_str_ned)
                .spawn()
                .expect("Unable to launch ffmpeg")
        };

        std::thread::sleep(Duration::from_secs(duration as u64) + Duration::from_secs(5u64 * 60));
        child.kill()
    });

    //ENG Stream
    std::thread::spawn(move || {
        let mut child = {
            Command::new("sh")
                .arg("-c")
                .arg(&ffmpeg_str_eng)
                .spawn()
                .expect("Unable to launch ffmpeg")
        };

        std::thread::sleep(Duration::from_secs(duration as u64) + Duration::from_secs(5u64 * 60));
        child.kill()
    }).join();
}