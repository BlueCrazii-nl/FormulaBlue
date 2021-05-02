use colored::Colorize;
use std::time::Duration;
use crate::apis::f1tv::{get_live_sessions, RetrieveItemsContainer};
use std::collections::HashMap;
use crate::apis::f1tv::playback::get_playback_url;
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
        ffmpeg::stream(url, 0, cfg);
    } else {
        refresh_races(cfg);
    }
}

const REFRESH_INTERVAL_SECONDS: u64 = 60;

fn refresh_races(cfg: Config) {
    std::thread::spawn(move || {
        'infinite_loop: loop {
            print!("Fetching live sessions...");
            let sessions = get_live_sessions();
            if sessions.is_err() {
                print!("{}\n", "FAIL".red());
                println!("{}", sessions.err().unwrap());

                std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
            } else {
                print!("{}\n", "OK".green());
                let sessions_unwrapped = sessions.unwrap();

                let mut final_sessions: HashMap<String, RetrieveItemsContainer> = HashMap::new();
                for ct in sessions_unwrapped {
                    if final_sessions.contains_key(&ct.id) {
                        continue;
                    }

                    final_sessions.insert(ct.id.clone(), ct.clone());
                }

                println!("Found {} live sessions!", final_sessions.len());
                let subscription_token: Option<String> = if final_sessions.len() > 0 {
                    let response_wrapped = apis::f1tv::login::do_login(&cfg.f1_username.clone().unwrap(), &cfg.f1_password.clone().unwrap());
                    if response_wrapped.is_err() {
                        print!("Failed to log in to F1TV. ");

                        match response_wrapped.err().unwrap().status {
                            Some(503) => {
                                println!("Got status code 503. Retrying in {} seconds.", REFRESH_INTERVAL_SECONDS);
                                continue 'infinite_loop;
                            },
                            Some(403) => {
                                println!("Got status code 403. Are your credentials correct? Exiting.");
                                std::process::exit(1);
                            },
                            _ => {
                                println!("Unknown error. Retrying in {} seconds.", REFRESH_INTERVAL_SECONDS);
                                std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
                                continue 'infinite_loop;
                            }
                        }
                    }

                    let response = response_wrapped.unwrap();
                    Some(response.data.subscription_token)
                } else {
                    None
                };

                for (_, v) in final_sessions {
                    let hls = get_playback_url(&subscription_token.clone().unwrap(), &v.id).expect("Failed to get HLS Stream");
                    println!("Starting FFMPEG streams.");
                    ffmpeg::stream(hls, v.metadata.duration, cfg.clone())
                }

                println!("Sleeping for {} seconds.", REFRESH_INTERVAL_SECONDS);
                std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
                continue;
            }
        }
    }).join().expect("");
}
