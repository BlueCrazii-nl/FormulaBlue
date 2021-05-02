use serde::{Serialize, Deserialize};
use crate::apis::f1tv::{LOGIN_ENDPOINT, API_KEY};

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

    #[serde(rename(deserialize = "data"))]
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

pub fn do_login(username: &str, password: &str) -> Result<SuccessfulLoginResponse, FailedLoginResponse> {
    let req = reqwest::blocking::Client::new()
        .post(LOGIN_ENDPOINT)
        .header("User-Agent", "RaceControl")
        .header("apiKey", API_KEY)
        .json(&LoginRequest { login: username, password })
        .send()
        .expect("An error occurred while sending a login request to F1TV.");

    return match req.status() {
        reqwest::StatusCode::OK => {
            let response: SuccessfulLoginResponse = req.json().expect("An error occurred while deserializing a login response");
            Ok(response)
        },
        reqwest::StatusCode::UNAUTHORIZED => {
            let mut response: FailedLoginResponse = req.json().expect("An error occurred while deserializing a login response");
            response.status = Some(401);
            Err(response)
        },
        _ => {
            let response = FailedLoginResponse {
                status: Some(req.status().as_u16()),
                detail: None
            };

            Err(response)
        }
    }
}