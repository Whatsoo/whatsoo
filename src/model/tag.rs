use sqlx::FromRow;
use sqlx::types::chrono::NaiveDateTime;
use crate::common::date_format;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub pk_id: u64,
    pub tag_name: String,
    pub uk_logo: String,
    pub parent_tag: u64,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
    pub create_user: u64,
    #[serde(with = "date_format")]
    pub update_time: NaiveDateTime,
    pub update_user: u64,
}