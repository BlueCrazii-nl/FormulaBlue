use std::io::Read;
use std::process::{Command, Stdio};
use crate::config::Config;
use std::time::Duration;
use log::{debug, info, trace, warn};

pub fn stream(session_id: String, _end_time: i64, cfg: Config) {
    let cfg_ned = cfg.clone();
    let cfg_eng = cfg.clone();
    let cfg_data = cfg.clone();

    //let end_time = end_time + 30i64 * 60i64;
    let end_time = time::OffsetDateTime::now_utc().unix_timestamp() + cfg.streams.stream_for;

    let subscription_token_ned = crate::apis::f1tv::login::get_subscription_token(&cfg).unwrap();
    let subscription_token_eng = subscription_token_ned.clone();
    let subscription_token_data = subscription_token_ned.clone();

    let session_id_ned = session_id.clone();
    let session_id_eng = session_id.clone();
    let session_id_data = session_id.clone();

    crossbeam_utils::thread::scope(|s| {
        if cfg.streams.ned {
            s.spawn(move |_| {
                info!("Starting NED Stream for session {}", session_id_ned);

                while time::OffsetDateTime::now_utc().unix_timestamp() < end_time {
                    let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_ned, &session_id_ned, None);

                    let ffmpeg_command = cfg_ned.commands.ned.as_deref().expect("Fetching NED FFMPEG command")
                        .replace("{source_url}", &hls_url.unwrap());

                    run_ffmpeg(&ffmpeg_command, end_time, "NED");
                }
            });
        }

        if cfg.streams.eng {
            s.spawn(move |_| {
                info!("Starting ENG Stream for session {}", session_id_eng);

                while chrono::Utc::now().timestamp() < end_time {
                    let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_eng, &session_id_eng, None);

                    //Construct the FFMPEG command for the NED stream
                    let ffmpeg_command = cfg_eng.commands.eng.as_deref().expect("Fetching ENG FFMPEG command")
                        .replace("{source_url}", &hls_url.unwrap());

                    run_ffmpeg(&ffmpeg_command, end_time, "ENG");
                }
            });
        }

        if cfg.streams.data {
            s.spawn(move |_| {
                info!("Starting data channel stream for session {}", session_id_data);

                while chrono::Utc::now().timestamp() < end_time {
                    let data_channel = match crate::apis::f1tv::get_data_channel(&session_id_data) {
                        Ok(x) => x,
                        Err(e) => {
                            warn!("There appears to be no data channel available: {}", e);
                            return;
                        }
                    };
                    let hls_url = crate::apis::f1tv::playback::get_playback_url(&subscription_token_data, &session_id_data, Some(&data_channel));

                    //Construct the FFMPEG command for the data stream
                    let ffmpeg_command = cfg_data.commands.data.as_deref().expect("Fetching DATA command")
                        .replace("{source_url}", &hls_url.unwrap());
                    run_ffmpeg(&ffmpeg_command, end_time, "DATA");
                }
            });
        }

    }).expect("Spawning threads");
}

fn run_ffmpeg(ffmpeg_command: &str, end_time: i64, source: &str) {
    let mut child = {
        Command::new("sh")
            .arg("-c")
            .arg(ffmpeg_command)
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

                if time::OffsetDateTime::now_utc().unix_timestamp() < end_time {
                    run_ffmpeg(ffmpeg_command, end_time, source);
                }
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

    if time::OffsetDateTime::now_utc().unix_timestamp() > end_time {
        // Kill FFMPEG if it hasnt killed itself yet
        match child.try_wait() {
            Ok(None) => child.kill().expect("Unable to kill ffmpeg"),
            _ => {}
        }
    } else {
        run_ffmpeg(ffmpeg_command, end_time, source);
    }

}