use crate::model::user::{User, RegisterUser};
use anyhow::Result;
use sqlx::MySqlPool;

pub async fn check_email_exists(email: String, pool: &MySqlPool) -> Result<i64> {
    User::check_email_exists(email, pool).await
}

pub async fn check_username_exists(username: String, pool: &MySqlPool) -> Result<i64> {
    User::check_username_exists(username, pool).await
}

pub async fn insert_one_user(user: RegisterUser, pool: &MySqlPool) -> Result<i64> {
    User::insert_one_user(user, pool).await
}