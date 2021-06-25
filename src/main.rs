use colored::Colorize;
use std::time::Duration;
use crate::apis::f1tv::get_live_sessions;
use crate::config::Config;

mod config;
mod apis;
mod ffmpeg;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("Starting FormulaBlue v{}", VERSION);
    print!("Reading configuration...");

    let cfg = config::read();
    let cfg_verify = cfg.verify();
    if !cfg_verify.0 {
        print!("{}\n", "FAIL".red());
        println!("Configuration did not pass checks: Field '{}' is empty.", cfg_verify.1);
        std::process::exit(1);
    } else {
        print!("{}\n", "OK".green());
    }

    print!("Attempting to log in to F1TV...");
    let cfg_cloned = cfg.clone();
    let login_response = apis::f1tv::login::do_login(&cfg_cloned.f1_username.unwrap(), &cfg_cloned.f1_password.unwrap());
    if login_response.is_err() {
        print!("{}\n", "FAIL".red());

        let err = login_response.err().unwrap();
        if err.detail.is_some() {
            println!("Login to F1TV failed. The reason is known as follows: '{}'", err.detail.unwrap());
        } else {
            println!("Login to F1TV failed. The reason is unknown. (status: {})", err.status.unwrap());
        }

        std::process::exit(1);
    } else {
        print!("{}\n", "OK".green());
    }

    if cfg.tmpurl.is_some() {
        let url = cfg.clone().tmpurl.unwrap();
        ffmpeg::stream(url, chrono::Utc::now().timestamp() + (60_i64 * 60_i64), cfg);
    } else {
        refresh_races(cfg);
    }
}

const REFRESH_INTERVAL_SECONDS: u64 = 10;

pub fn refresh_races(cfg: Config) {
    std::thread::spawn(move || {
        let mut running_session_end_time: i64 = 0;

        loop {
            print!("Fetching live sessions...");
            let live_sessions = get_live_sessions();
            if live_sessions.is_err() {
                print!("{}\n", "FAIL".red());
                println!("{}", live_sessions.err().unwrap());

                std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
            } else {
                print!("{}\n", "OK".green());
                let live_sessions = live_sessions.unwrap();

                println!("Found {} live sessions!", live_sessions.len());
                println!("Starting FFMPEG streams.");

                for session in live_sessions.clone() {
                    let emf_attr = session.metadata.unwrap().emf_attributes.unwrap();
                    running_session_end_time = emf_attr.session_end_date + (30_i64 * 60_i64);
                    ffmpeg::stream(session.id.unwrap(), emf_attr.session_end_date, cfg.clone());
                }

                let sleep_time = {
                    if live_sessions.len() > 0 {
                        running_session_end_time - chrono::Utc::now().timestamp()
                    } else {
                        REFRESH_INTERVAL_SECONDS as i64
                    }
                };

                println!("Sleeping for {} seconds.", sleep_time);
                std::thread::sleep(Duration::from_secs(sleep_time as u64));
                continue;
            }
        }
    }).join().expect("");
}



