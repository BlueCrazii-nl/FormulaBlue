use std::io::Read;
use std::process::{Command, Stdio};
use crate::config::Config;
use std::time::Duration;
use log::{debug, info, trace};

pub fn stream(session_id: String, end_time: i64, cfg: Config) {
    let cfg_ned = cfg.clone();
    let cfg_eng = cfg.clone();
    let cfg_data = cfg.clone();

    let end_time = end_time + 30i64 * 60i64;

    let subscription_token_ned = crate::apis::f1tv::login::get_subscription_token(&cfg).unwrap();
    let subscription_token_eng = subscription_token_ned.clone();
    let subscription_token_data = subscription_token_ned.clone();

    let session_id_ned = session_id.clone();
    let session_id_eng = session_id.clone();
    let session_id_data = session_id.clone();

    //NED loop
    std::thread::spawn(move || {
        info!("Starting NED Stream for session {}", session_id_ned);

        while time::OffsetDateTime::now_utc().unix_timestamp() < end_time {
            let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_ned, &session_id_ned, None);

            let ffmpeg_command = cfg_ned.ffmpeg_command
                .replace("{source_url}", &hls_url.unwrap())
                .replace("{ingest}", &cfg_ned.rtmp_ingest)
                .replace("{lang}", "nld")
                .replace("{key}", &cfg_ned.ned_key);

            run_ffmpeg(ffmpeg_command, end_time, "NED");
        }
    });

    //ENG loop
    std::thread::spawn(move || {
        info!("Starting ENG Stream for session {}", session_id_eng);

        while chrono::Utc::now().timestamp() < end_time {
            let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_eng, &session_id_eng, None);

            //Construct the FFMPEG command for the NED stream
            let ffmpeg_command = cfg_eng.ffmpeg_command
                .replace("{source_url}", &hls_url.unwrap())
                .replace("{ingest}", &cfg_eng.rtmp_ingest)
                .replace("{lang}", "eng")
                .replace("{key}", &cfg_eng.eng_key);

            run_ffmpeg(ffmpeg_command, end_time, "ENG");
        }
    });

    //DATA CHANNEL
    std::thread::spawn(move || {
        info!("Starting data channel stream for session {}", session_id_data);

        while chrono::Utc::now().timestamp() < end_time {
            let data_channel = crate::apis::f1tv::get_data_channel(&session_id_data).expect("Failed to get Data channel ID");
            let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_data, &session_id_data, Some(&data_channel));

            //Construct the FFMPEG command for the data stream
            let ffmpeg_command = cfg_data.data_command
                .replace("{source_url}", &hls_url.unwrap())
                .replace("{ingest}", &cfg_data.rtmp_ingest)
                .replace("{lang}", "eng")
                .replace("{key}", &cfg_data.data_key);
            run_ffmpeg(ffmpeg_command, end_time, "DATA");
        }
    }).join().unwrap();
}

fn run_ffmpeg(ffmpeg_command: String, end_time: i64, source: &str) {
    let mut child = {
        Command::new("sh")
            .arg("-c")
            .arg(&ffmpeg_command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Unable to launch ffmpeg")
    };

    let mut exit = None;
    while time::OffsetDateTime::now_utc().unix_timestamp() < end_time {
        //Check if the command has exited
        match child.try_wait() {
            Ok(Some(e)) => {
                // FFMPEG has exited
                exit = Some(e);
                break;
            },
            _ => {}
        }
        std::thread::sleep(Duration::from_secs(5));
    }

    if let Some(exit) = exit {
        debug!("FFMPEG stream {} exited. Exit code: {:?}", source, exit.code());

        let mut stdout = String::default();
        let mut stderr = String::default();
        child.stdout.take().expect("Missing stdout").read_to_string(&mut stdout).expect("Failed to read stdout");
        child.stderr.take().expect("Missing stderr").read_to_string(&mut stderr).expect("Failed to read stderr");

        trace!("Stdout: {}", stdout);
        trace!("Stderr: {}", stderr);
    } else { unreachable!() }

    // Kill FFMPEG if it hasnt killed itself yet
    match child.try_wait() {
        Ok(None) => child.kill().expect("Unable to kill ffmpeg"),
        _ => {}
    }
}