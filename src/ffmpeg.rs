use std::process::Command;
use crate::config::Config;
use std::time::Duration;
use std::sync::mpsc::TryRecvError;

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

    let end_time = chrono::Utc::now() + (chrono::Duration::seconds(duration) + chrono::Duration::seconds(5i64 * 60));

    //Create channels so both threads can signal each other when they exit
    let (tx_ned, rx_ned) = std::sync::mpsc::channel();
    let (tx_eng, rx_eng) = std::sync::mpsc::channel();

    //NED Stream
    std::thread::spawn(move || {
        let mut child = {
            Command::new("sh")
                .arg("-c")
                .arg(&ffmpeg_str_ned)
                .spawn()
                .expect("Unable to launch ffmpeg")
        };

        while chrono::Utc::now() < end_time {
            //Check if the command has exited
            match child.try_wait() {
                Ok(Some(_)) => break,
                _ => {}
            }
            //Check if the other thread has exited
            match rx_eng.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => break,
                _ => {}
            }

            std::thread::sleep(Duration::from_secs(5));
        }
        child.kill();
        //Signal to other thread to stop the command
        tx_ned.send(1).unwrap();
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

        while chrono::Utc::now() < end_time {
            //Check if the command has exited
            match child.try_wait() {
                Ok(Some(_)) => break,
                _ => {}
            }
            //Check if the other thread has exited
            match rx_ned.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => break,
                _ => {}
            }

            std::thread::sleep(Duration::from_secs(5));
        }

        child.kill();
        //Signal to the other thread to stop the command
        tx_eng.send(1).unwrap();
    }).join();
}