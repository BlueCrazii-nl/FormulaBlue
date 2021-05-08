use std::process::Command;
use crate::config::Config;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

pub fn stream(url: String, end_time: i64, cfg: Config) {
    let ffmpeg_str = cfg.ffmpeg_command.unwrap();
    let ingest_url = cfg.rtmp_ingest.unwrap();

    //Construct the FFMPEG command for the NED stream
    let ffmpeg_str_ned = ffmpeg_str
        .replace("{source_url}", &url)
        .replace("{ingest}", &ingest_url)
        .replace("{lang}", "nld")
        .replace("{key}", &cfg.ned_key.unwrap());

    //Construct the FFMPEG command for the ENG stream
    let ffmpeg_str_eng = ffmpeg_str
        .replace("{source_url}", &url)
        .replace("{ingest}", &ingest_url)
        .replace("{lang}", "eng")
        .replace("{key}", &cfg.eng_key.unwrap());

    //We let the stream play 10 minutes longer than its session end time
    let adjusted_end_time = end_time + (10_i64 * 60_i64);

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

        //As long as the current time is less than our planned end time
        //we want to check if ffmpeg has failed (usually caused by an issue with F1TV)
        //If so, we want to restart it
        while chrono::Utc::now().timestamp() < adjusted_end_time {
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

        child.kill().expect("Unable to kill ffmpeg");

        //Signal to other thread to stop the command
        tx_ned.send(1).unwrap_or_else(|_| eprintln!("Unable to send a message to the ENG thread to stop."));
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

        //As long as the current time is less than our planned end time
        //we want to check if ffmpeg has failed (usually caused by an issue with F1TV)
        //If so, we want to restart it
        while chrono::Utc::now().timestamp() > adjusted_end_time {
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

        child.kill().expect("Unable to kill ffmpeg");

        //Signal to the other thread to stop the command
        tx_eng.send(1).unwrap_or_else(|_| eprintln!("Unable to send a message to the NED thread to stop."));
    }).join().unwrap();
}