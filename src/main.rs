use colored::Colorize;
use chrono::Datelike;
use std::time::Duration;

mod config;
mod apis;
mod race_tracker;

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
    let login_response = apis::f1tv::login::do_login(&cfg.f1_username.unwrap(), &cfg.f1_password.unwrap());
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

    refresh_races();
}

const REFRESH_INTERVAL_SECONDS: u64 = 60;

fn refresh_races() {
    std::thread::spawn(move || {
        let mut discovered_session_ids: Vec<String> = Vec::new();
        loop {
            print!("Fetching F1 {} Meetings...", chrono::Utc::now().year());
            let events = apis::f1tv::get_meetings();
            let events = if events.is_err() {
                print!("{}\n", "FAIL".red());
                println!("{:?}", events.err().unwrap());

                std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
                continue;
            } else {
                print!("{}\n", "OK".green());
                events.unwrap()
            };

            let mut new_events = 0;
            for event in events.result_obj.containers {
                if event.metadata.emf_attributes.meeting_key.is_empty() {
                    continue;
                }

                if discovered_session_ids.contains(&event.id) {
                    continue;
                }

                new_events += 1;
                print!("Fetching Live Sessions for '{}' with meeting key '{}' and id '{}'...", event.metadata.emf_attributes.meeting_name, &event.metadata.emf_attributes.meeting_key, &event.id);
                let live_sessions = apis::f1tv::get_live_sessions(&event.metadata.emf_attributes.meeting_key);

                if live_sessions.is_err() {
                    print!("{}\n", "FAIL".red());
                    println!("{:?}", live_sessions.err().unwrap());
                    std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
                    continue;
                } else {
                    print!("{}\n", "OK".green());
                    let live_sessions = live_sessions.unwrap();
                    discovered_session_ids.push(event.id);

                    if live_sessions.len() > 0 {
                        //TODO Fetch live session HLS url and start streaming to Blue's ingress
                        //Should do that on a different thread, and keep track of if we're already streaming.
                    }
                }

                //Every 50 items we'll sleep for 10 seconds, as to avoid hitting rate limits.
                if new_events % 50 == 0 {
                    print!("Sleeping for 10 seconds to avoid hitting F1TV rate limits...");
                    std::thread::sleep(Duration::from_secs(10));
                    print!("{}\n", "OK".green());
                }
            }

            println!("Retrieved {} new events. Sleeping for {} seconds.", new_events, REFRESH_INTERVAL_SECONDS);
            std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));

        }
    }).join();
}
