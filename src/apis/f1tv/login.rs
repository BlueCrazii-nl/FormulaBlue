use serde::{Serialize, Deserialize};
use crate::apis::f1tv::{LOGIN_ENDPOINT, API_KEY};
use crate::config::Config;
use std::time::Duration;
use anyhow::Result;
use log::warn;
use thiserror::Error;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct LoginRequest<'a> {
    login:      &'a str,
    password:   &'a str
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct SuccessfulLoginResponse {
    pub session_id:             String,
    pub password_is_temporary:  bool,
    pub subscriber:             Subscriber,
    pub country:                String,
    #[serde(rename = "data")]
    pub data:                   SubscriptionData
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Subscriber {
    pub first_name:     String,
    pub last_name:      String,
    pub home_country:   String,
    pub id:             i64,
    pub email:          String,
    pub login:          String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionData {
    pub subscription_status:    String,
    pub subscription_token:     String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct FailedLoginResponse {
    pub status: Option<u16>,
    pub detail: Option<String>
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Reqwest error: {0:?}")]
    Reqwest(#[from] reqwest::Error),
    #[error("You are unauthorized")]
    Unauthorized,
    #[error("Unknown error")]
    Unknown,
    #[error("Service is temporarily unavailable")]
    ServiceUnavailable
}

pub fn do_login(username: &str, password: &str) -> std::result::Result<SuccessfulLoginResponse, LoginError> {
    let req = reqwest::blocking::Client::new()
        .post(LOGIN_ENDPOINT)
        .header("User-Agent", "RaceControl")
        .header("apiKey", API_KEY)
        .json(&LoginRequest { login: username, password })
        .send()?;

    return match req.status() {
        reqwest::StatusCode::OK => {
            let response: SuccessfulLoginResponse = req.json()?;
            Ok(response)
        },
        reqwest::StatusCode::SERVICE_UNAVAILABLE => Err(LoginError::ServiceUnavailable.into()),
        reqwest::StatusCode::UNAUTHORIZED => Err(LoginError::Unauthorized.into()),
        _ => Err(LoginError::Unknown.into())
    }
}

pub fn get_subscription_token(cfg: &Config) -> Result<String> {
    match do_login(&cfg.f1tv.username, &cfg.f1tv.password) {
        Ok(l) => Ok(l.data.subscription_token),
        Err(e) => {
            match e {
                LoginError::ServiceUnavailable => {
                    warn!("Got status code 503 while attempting to log in. Retrying in 5 seconds");
                    std::thread::sleep(Duration::from_secs(5));
                    get_subscription_token(cfg)
                },
                _ => Err(e.into())
            }
        }
    }
}