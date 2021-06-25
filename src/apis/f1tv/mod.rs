use serde::Deserialize;
use std::collections::HashMap;

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
    pub id: Option<String>,
    pub metadata: Metadata,
    pub properties: Option<Vec<Properties>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Properties {
    pub series: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Metadata {
    #[serde(rename(deserialize = "emfAttributes"))]
    pub emf_attributes: Option<EmfAttributes>,
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
    pub meeting_key:    String,
    #[serde(rename(deserialize = "sessionEndDate"))]
    pub session_end_date: i64
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

            if ct_inner.metadata.emf_attributes.is_none() {

            }

            if ct_inner.properties.is_some() && ct_inner.properties.clone().unwrap().get(0).unwrap().series != "FORMULA 1" {
                continue;
            }

            if ct_inner.metadata.object_type == "VIDEO" && ct_inner.metadata.content_sub_type.clone().unwrap() == "LIVE" {
                live_responses.push(ct_inner)
            }
        }
    }

    let mut live_sessions_filtered: HashMap<String, RetrieveItemsContainer> = HashMap::new();
    for ct in live_responses {
        if ct.id.is_some() && live_sessions_filtered.contains_key(&ct.id.clone().unwrap()) {
            continue;
        }

        live_sessions_filtered.insert(ct.id.clone().unwrap(), ct.clone());
    }

    //Turn the HashMap into a vec of it's values
    let mut result = Vec::new();
    for (_, v) in live_sessions_filtered {
        result.push(v);
    }

    Ok(result)
}