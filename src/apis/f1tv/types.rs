use serde::Deserialize;

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