use serde::Deserialize;
use crate::apis::f1tv::BASE_URL;

const ENDPOINT_PATH: &str = "/1.0/R/ENG/BIG_SCREEN_HLS/ALL/CONTENT/PLAY";

#[derive(Deserialize)]
struct PlaybackResponse {
    #[serde(rename(deserialize = "resultObj"))]
    result_obj: ResultObj
}

#[derive(Deserialize)]
struct ResultObj {
    url: String
}

pub fn get_playback_url(subscription_token: &str, content_id: &str) -> reqwest::Result<String> {
    let req: PlaybackResponse = reqwest::blocking::Client::new().get(format!("{}{}", BASE_URL, ENDPOINT_PATH))
        .query(&[("contentId", content_id)])
        .header("ascendontoken", subscription_token)
        .send()?
        .json()?;

    Ok(req.result_obj.url)
}