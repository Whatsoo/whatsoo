use sqlx::types::chrono::NaiveDateTime;
use crate::common::date_format;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Star {
    pub pk_id: u64,
    pub star_type: StarType,
    pub user_id: u64,
    pub star_id: u64,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
}

#[derive(Debug, Copy, Clone,Serialize, Deserialize)]
pub enum StarType {
    User = 1,
    Topic = 2,
    Comment = 3,
}