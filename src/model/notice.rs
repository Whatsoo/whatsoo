use sqlx::types::chrono::NaiveDateTime;
use crate::common::date_format;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notice {
    pub pk_id: u64,
    pub content: String,
    pub notified_user_id: u64,
    pub viewed: bool,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
    pub create_user: u64,
}