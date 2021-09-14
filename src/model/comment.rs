use crate::common::date_format;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub pk_id: u64,
    pub user_id: u64,
    pub topic_id: u64,
    pub content: String,
    pub like_amount: u64,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
    pub create_user: u64,
}
