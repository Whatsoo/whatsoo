use crate::model::user::User;
use anyhow::Result;
use sqlx::MySqlPool;

pub async fn check_email_exists(email: String, pool: &MySqlPool) -> Result<i64> {
    User::check_email_exists(email, pool).await
}

pub async fn check_username_exists(username: String, pool: &MySqlPool) -> Result<i64> {
    User::check_username_exists(username, pool).await
}
