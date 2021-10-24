use std::process::{Command, Stdio};
use crate::config::Config;
use std::time::Duration;

pub fn stream(session_id: String, end_time: i64, cfg: Config) {
    let cfg_1 = cfg.clone();
    let cfg_2 = cfg.clone();
    let cfg_3 = cfg.clone();

    let ffmpeg_str = cfg.ffmpeg_command.unwrap();
    let ingest_url = cfg.rtmp_ingest.unwrap();

    let ffmpeg_str_1 = ffmpeg_str.clone();
    let ingest_url_1 = ingest_url.clone();

    let ffmpeg_str_2 = cfg_3.clone().data_command.unwrap();
    let ingest_url_2 = ingest_url.clone();

    let end_time = end_time + 30i64 * 60i64;

    let subscription_token = crate::apis::f1tv::login::get_subscription_token(cfg_1.clone()).unwrap();
    let subscription_token_1 = subscription_token.clone();
    let subscription_token_2 = subscription_token.clone();

    let session_id_1 = session_id.clone();
    let session_id_2 = session_id.clone();

    //NED loop
    std::thread::spawn(move || {
        println!("Starting NED Stream");

        while chrono::Utc::now().timestamp() < end_time {
            let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token, &session_id_1, None);

            //Construct the FFMPEG command for the NED stream
            let ffmpeg_command = ffmpeg_str_1
                .replace("{source_url}", &hls_url.unwrap())
                .replace("{ingest}", &ingest_url_1)
                .replace("{lang}", "nld")
                .replace("{key}", &cfg_1.clone().ned_key.unwrap());

            run_ffmpeg(ffmpeg_command, end_time, "NED");
        }
    });

    //ENG loop
    std::thread::spawn(move || {
        println!("Starting ENG Stream");

        while chrono::Utc::now().timestamp() < end_time {
            let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_1, &session_id, None);

            //Construct the FFMPEG command for the NED stream
            let ffmpeg_command = ffmpeg_str.clone()
                .replace("{source_url}", &hls_url.unwrap())
                .replace("{ingest}", &ingest_url.clone())
                .replace("{lang}", "eng")
                .replace("{key}", &cfg_2.clone().eng_key.unwrap());

            run_ffmpeg(ffmpeg_command, end_time, "ENG");
        }
    });

    //DATA CHANNEL
    std::thread::spawn(move || {
        println!("Starting data channel stream");

        while chrono::Utc::now().timestamp() < end_time {
            let data_channel = crate::apis::f1tv::get_data_channel(&session_id_2).expect("Failed to get Data channel ID");
            let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_2, &session_id_2, Some(&data_channel));

            //Construct the FFMPEG command for the data stream
            let ffmpeg_command = ffmpeg_str_2.clone()
                .replace("{source_url}", &hls_url.unwrap())
                .replace("{ingest}", &ingest_url_2.clone())
                .replace("{lang}", "eng")
                .replace("{key}", &cfg_3.clone().data_key.unwrap());
            run_ffmpeg(ffmpeg_command, end_time, "DATA");
        }
    }).join().unwrap();
}

fn run_ffmpeg(ffmpeg_command: String, end_time: i64, source: &str) {
    println!("Starting ffmpeg stream '{}'", source);

    let mut child = {
        Command::new("sh")
            .arg("-c")
            .arg(&ffmpeg_command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Unable to launch ffmpeg")
    };


    let exit_status = while chrono::Utc::now().timestamp() < end_time {
        //Check if the command has exited
        match child.try_wait() {
            Ok(Some(e)) => break e,
            _ => {}
        }

        std::thread::sleep(Duration::from_secs(5));
    };

    println!("FFmpeg has exited for {} with code {}", source, child);
    println!("Stdout: {:?}", child.stdout);
    println!("Stderr: {:?}", child.stderr);

    match child.try_wait() {
        Ok(None) => child.kill().expect("Unable to kill ffmpeg"),
        _ => {}
    }
}