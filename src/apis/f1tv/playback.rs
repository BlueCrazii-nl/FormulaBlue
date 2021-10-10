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

pub fn get_playback_url(subscription_token: &str, content_id: &str, channel: Option<&str>) -> reqwest::Result<String> {
    let mut query = vec![(("contentId", content_id))];
    if let Some(channel) = channel {
        query.push(("channelId", channel));
    }

    let req: PlaybackResponse = reqwest::blocking::Client::new().get(format!("{}{}", BASE_URL, ENDPOINT_PATH))
        .query(query.as_slice())
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.159 Safari/537.36 Edg/92.0.902.78")
        .header("ascendontoken", subscription_token)
        .send()?
        .json()?;

    Ok(req.result_obj.url)
}