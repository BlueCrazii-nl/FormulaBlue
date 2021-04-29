use chrono::Datelike;
use serde::Deserialize;

pub mod login;
pub mod playback;

pub const API_KEY: &str = "fCUCjWrKPu9ylJwRAv8BpGLEgiAuThx7";

pub const LOGIN_ENDPOINT: &str = "https://api.formula1.com/v2/account/subscriber/authenticate/by-password";
pub const BASE_URL: &str = "https://f1tv.formula1.com";
pub const DEFAULT_STREAM_TYPE: &str = "BIG_SCREEN_HLS";

pub const VOD_URL: &str = "/2.0/R/ENG/BIG_SCREEN_HLS/ALL/PAGE/SEARCH/VOD/F1_TV_Pro_Annual/2";


#[derive(Deserialize, Debug)]
pub struct EventLiveSessionsResponse {
    #[serde(rename(deserialize = "resultObj"))]
    pub result_obj: ResultObject<LiveRacesContainer>
}

#[derive(Deserialize, Debug)]
pub struct ResultObject<T> {
    pub containers: Vec<T>
}

#[derive(Deserialize, Debug)]
pub struct OuterContainer<T> {
    pub containers: ResultObject<T>
}

#[derive(Deserialize, Debug)]
pub struct LiveRacesContainer {
    pub id:         String,
    pub metadata:   Metadata
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub title:      String,

    #[serde(default)]
    #[serde(rename(deserialize = "objectType"))]
    pub object_type:    ObjectType,
    pub duration:       u32,

    #[serde(rename(deserialize = "emfAttributes"))]
    pub emf_attributes: EmfAttributes
}

#[derive(Deserialize, Debug)]
pub struct SeasonMeetingResponse {
    #[serde(rename(deserialize = "resultObj"))]
    pub result_obj: ResultObject<SeasonEventContainer>
}

#[derive(Deserialize, Debug)]
pub struct SeasonEventContainer {
    pub id:         String,
    pub metadata:   Metadata
}

#[derive(Deserialize, Debug)]
pub struct EmfAttributes {
    #[serde(rename(deserialize = "MeetingKey"))]
    pub meeting_key:    String,

    #[serde(rename(deserialize = "Meeting_Name"))]
    pub meeting_name:   String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum ObjectType {
    Video,
    Live,
    Bundle,
    Image,
    Unknown
}

impl Default for ObjectType {
    fn default() -> Self {
        Self::Unknown
    }
}

pub fn get_live_sessions(meeting_key: &str) -> reqwest::Result<Vec<LiveRacesContainer>> {
    let req: EventLiveSessionsResponse = reqwest::blocking::Client::new().get(format!("{}{}", BASE_URL, VOD_URL))
        .query(&[
            ("orderBy", "meeting_End_Date"),
            ("sortOrder", "asc"),
            ("filter_MeetingKey", meeting_key),
            ("filter_orderByFom", "Y"),
            ("maxResults", "100")])
        .send()?
        .json()?;
        //.text()?;

    //println!("{:?}", req);
    //std::process::exit(1);

    let mut live_responses: Vec<LiveRacesContainer> = Vec::new();
    for ct in req.result_obj.containers {
        match ct.metadata.object_type {
            ObjectType::Live => live_responses.push(ct),
            _ => continue
        }
    }


    //Ok(vec![LiveRacesContainer { metadata: Metadata { object_type: ObjectType::Live, emf_attributes: EmfAttributes { meeting_name: "".to_string(), meeting_key: "".to_string()}, title: "".to_string(), duration: 0}, id: "".to_string()}])

    Ok(live_responses)
}

pub fn get_meetings() -> reqwest::Result<SeasonMeetingResponse> {
    let req = reqwest::blocking::Client::new().get(format!("{}{}", BASE_URL, VOD_URL))
        .query(&[
            ("orderBy", "meeting_Number"),
            ("sort_by", "asc"),
            ("filter_objectSubType", "Meeting"),
            ("filter_season", &chrono::Utc::now().year().to_string()),
            ("filter_orderByFom", "Y"),
            ("maxResults", "100")
        ])
        .header("User-Agent", "FormulaBlue")
        .send()?
        .json()?;

    Ok(req)
}