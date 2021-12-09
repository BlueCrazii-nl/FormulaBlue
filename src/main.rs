use std::process::exit;
use std::time::Duration;
use crate::apis::f1tv::get_live_sessions;
use crate::config::Config;
use log::{warn, info, debug, error};

mod config;
mod apis;
mod ffmpeg;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REFRESH_INTERVAL_SECONDS: u64 = 10;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{}=INFO", env!("CARGO_PKG_NAME")));
    }
    env_logger::init();

    info!("Starting FormulaBlue v{}", VERSION);
    info!("Reading configuration...");
    let config = match Config::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to read configuration file: {:?}", e);
            exit(1);
        }
    };

    info!("Logging in to F1TV");
    match apis::f1tv::login::do_login(&config.f1_username, &config.f1_password) {
        Ok(_) => debug!("Logged in"),
        Err(e) => {
            error!("Failed to log in to F1TV: {}", e);
            exit(1);
        }
    }

    if let Some(ref _test_session_id) = config.test_session_id {
        warn!("Test session ID was supplied. Running in test mode!");
        todo!("This has not been implemented yet");
    }

    loop {
        info!("Fetching live sessions");
        let live_sessions = match get_live_sessions() {
            Ok(l) => l,
            Err(e) => {
                warn!("Failed to fetch live sessions, trying again later: {:?}", e);
                std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
                continue;
            }
        };
        debug!("Fetched successfully");
        info!("Found {} live sessions", live_sessions.len());
        if live_sessions.is_empty() {
            info!("Sleeping for {} seconds", REFRESH_INTERVAL_SECONDS);
            std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
            continue;
        }

        debug!("Requesting subscription token");
        let token = match crate::apis::f1tv::login::get_subscription_token(&config) {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to fetch subscription token: {:?}. Retrying again later", e);
                std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
                continue;
            }
        };
        debug!("Got subscription token {}", &token);
        let session = live_sessions.first().unwrap(); // Safe due to is_empty() check above

        let metadata = session.metadata.as_ref().unwrap();
        let emf_attr = metadata.emf_attributes.as_ref().unwrap();
        let end_time = time::OffsetDateTime::from_unix_timestamp(emf_attr.session_end_date).expect("Unable to convert end timestamp to OffsetDateTime").checked_add(time::Duration::minutes(30)).expect("Unable to add Duration");
        debug!("Session ends at {}-{}-{} {}:{}:{}", end_time.year(), end_time.month(), end_time.day(), end_time.hour(), end_time.minute(), end_time.second());

        debug!("Starting FFMPEG streams");
        ffmpeg::stream(session.id.as_ref().unwrap().to_string(), emf_attr.session_end_date, config.clone());


        let sleep_time = (end_time - time::OffsetDateTime::now_utc()).whole_seconds();
        info!("Sleeping main thread for {} seconds.", sleep_time);
        std::thread::sleep(Duration::from_secs(sleep_time as u64));
    }
}