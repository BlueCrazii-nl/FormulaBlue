use std::process::Command;
use crate::config::Config;
use std::time::Duration;

pub fn stream(session_id: String, end_time: i64, cfg: Config) {
    let cfg_1 = cfg.clone();
    let cfg_2 = cfg.clone();

    let ffmpeg_str = cfg.ffmpeg_command.unwrap();
    let ingest_url = cfg.rtmp_ingest.unwrap();

    let ffmpeg_str_1 = ffmpeg_str.clone();
    let ingest_url_1 = ingest_url.clone();

    let end_time = end_time + 30i64 * 60i64;

    let subscription_token = crate::apis::f1tv::login::get_subscription_token(cfg_1.clone()).unwrap();
    let subscription_token_1 = crate::apis::f1tv::login::get_subscription_token(cfg_1.clone()).unwrap();

    //NED loop
    let session_id_1 = session_id.clone();
    std::thread::spawn(move || {
        while chrono::Utc::now().timestamp() < end_time {
            let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token, &session_id_1);

            //Construct the FFMPEG command for the NED stream
            let ffmpeg_command = ffmpeg_str_1
                .replace("{source_url}", &hls_url.unwrap())
                .replace("{ingest}", &ingest_url_1)
                .replace("{lang}", "nld")
                .replace("{key}", &cfg_1.clone().ned_key.unwrap());

            run_ffmpeg(ffmpeg_command, end_time);
        }
    });

    //ENG loop
    std::thread::spawn(move || {
        println!("Starting ENG Stream");

        while chrono::Utc::now().timestamp() < end_time {
            let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_1, &session_id);

            //Construct the FFMPEG command for the NED stream
            let ffmpeg_command = ffmpeg_str.clone()
                .replace("{source_url}", &hls_url.unwrap())
                .replace("{ingest}", &ingest_url.clone())
                .replace("{lang}", "eng")
                .replace("{key}", &cfg_2.clone().eng_key.unwrap());

            run_ffmpeg(ffmpeg_command, end_time);
        }
    }).join().unwrap();
}

fn run_ffmpeg(ffmpeg_command: String, end_time: i64) {
    let mut child = {
        Command::new("sh")
            .arg("-c")
            .arg(&ffmpeg_command)
            .spawn()
            .expect("Unable to launch ffmpeg")
    };

    while chrono::Utc::now().timestamp() < end_time {
        //Check if the command has exited
        match child.try_wait() {
            Ok(Some(_)) => break,
            _ => {}
        }

        std::thread::sleep(Duration::from_secs(5));
    }

    match child.try_wait() {
        Ok(None) => child.kill().expect("Unable to kill ffmpeg")
        _ => {}
    }
}