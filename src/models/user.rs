use chrono::{DateTime, Utc};
use http::Uri;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserMode {
    Normal,
    Observer,
    Receiver,
    Benefactor,
    Bot,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub short_name: String,
    pub full_name: String,
    pub display_name: String,
    pub username: String,
    pub email: String,
    pub path: String,
    #[serde(with = "http_serde::uri")]
    pub full_pic_url: Uri,
    #[serde(with = "http_serde::uri")]
    pub profile_pic_url: Uri,
    pub first_name: String,
    pub last_name: Option<String>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub budget_boost: u64,
    pub user_mode: UserMode,
    pub country: Option<String>,
    pub time_zone: String,
    pub can_receive: bool,
    pub can_give: bool,
    pub give_amounts: Vec<u32>,
    pub custom_properties: serde_json::Map<String, serde_json::Value>,
    pub status: String,
    pub earning_balance: Option<u64>,
    pub earning_balance_with_currency: Option<String>,
    pub lifetime_earnings: Option<u64>,
    pub lifetime_earnings_with_currency: Option<String>,
    pub admin: Option<bool>,
}
