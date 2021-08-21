use crate::common::date_format;
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Topic {
    pub pk_id: u64,
    pub user_id: u64,
    pub title: String,
    pub content: String,
    pub tags: String,
    pub like_times: u32,
    pub click_times: u32,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
    pub create_user: u64,
    #[serde(with = "date_format")]
    pub update_time: NaiveDateTime,
    pub update_user: u64,
}
