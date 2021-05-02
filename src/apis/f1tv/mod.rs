use serde::Deserialize;

pub mod login;
pub mod playback;

pub const API_KEY: &str = "fCUCjWrKPu9ylJwRAv8BpGLEgiAuThx7";

pub const LOGIN_ENDPOINT: &str = "https://api.formula1.com/v2/account/subscriber/authenticate/by-password";
pub const BASE_URL: &str = "https://f1tv.formula1.com";
pub const DEFAULT_STREAM_TYPE: &str = "BIG_SCREEN_HLS";

#[derive(Deserialize, Debug, Clone)]
pub struct ResultObject<T> {
    pub containers: Vec<T>
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiveSessionResponse {
    #[serde(rename(deserialize = "resultObj"))]
    pub result_obj: ResultObject<LiveSessionContainer>
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiveSessionContainer {
    #[serde(rename(deserialize = "retrieveItems"))]
    pub retrieve_items: RetrieveItems
}

#[derive(Deserialize, Debug, Clone)]
pub struct RetrieveItems {
    #[serde(rename(deserialize = "resultObj"))]
    pub result_obj: ResultObject<RetrieveItemsContainer>
}

#[derive(Deserialize, Debug, Clone)]
pub struct RetrieveItemsContainer {
    pub id: String,
    pub metadata: Metadata
}

#[derive(Deserialize, Debug, Clone)]
pub struct Metadata {
    #[serde(rename(deserialize = "emfAttributes"))]
    pub emf_attributes: EmfAttributes,
    #[serde(rename(deserialize = "longDescription"))]
    pub long_description: String,
    #[serde(rename(deserialize = "objectType"))]
    pub object_type: String,
    #[serde(rename(deserialize = "contentSubtype"))]
    pub content_sub_type: Option<String>,
    pub duration: i64
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmfAttributes {
    #[serde(rename(deserialize = "Meeting_Name"))]
    pub meeting_name:   String,
    #[serde(rename(deserialize = "MeetingKey"))]
    pub meeting_key:    String
}

pub fn get_live_sessions() -> reqwest::Result<Vec<RetrieveItemsContainer>> {
    let req: LiveSessionResponse = reqwest::blocking::Client::new().get(format!("{}/2.0/R/ENG/{DEFAULT_STREAM_TYPE}/ALL/PAGE/395/F1_TV_Pro_Annual/2", BASE_URL, DEFAULT_STREAM_TYPE = DEFAULT_STREAM_TYPE))
        .send()?
        .json()?;


    let mut live_responses: Vec<RetrieveItemsContainer> = Vec::new();
    for ct_outer in req.result_obj.containers {
        'inner: for ct_inner in ct_outer.retrieve_items.result_obj.containers {
            if ct_inner.metadata.content_sub_type.is_none() {
                continue 'inner;
            }

            if ct_inner.metadata.object_type == "VIDEO" && ct_inner.metadata.content_sub_type.clone().unwrap() == "LIVE" {
                live_responses.push(ct_inner)
            }
        }
    }

    Ok(live_responses)
}