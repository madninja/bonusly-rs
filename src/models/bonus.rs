use super::User;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Bonus {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub parent_bonus_id: Option<String>,
    pub reason: String,
    pub reason_decoded: String,
    pub reason_html: String,
    pub amount: u32,
    pub amount_with_currency: String,
    pub family_amount: u32,
    pub value: Option<String>,
    pub hashtag: Option<String>,
    pub giver: User,
    pub receivers: Vec<User>,
}
