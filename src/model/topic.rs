use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::common::date_format;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Topic {
    pub pk_id: u64,
    pub user_id: u64,
    pub title: String,
    pub content: String,
    pub tags: String,
    pub like_times: u64,
    pub click_times: u64,
    pub top: bool,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
    pub create_user: u64,
    #[serde(with = "date_format")]
    pub update_time: NaiveDateTime,
    pub update_user: u64,
}

#[derive(Debug, Deserialize)]
pub struct TopicFront {
    pub user_id: Option<u64>,
    pub title: String,
    pub content: String,
    pub tags: String,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub current_page: u32,
    pub page_size: u32,
}
