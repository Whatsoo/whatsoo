use crate::model::user::User;
use anyhow::Result;
use sqlx::MySqlPool;

pub async fn find_all(pool: &MySqlPool) -> Result<Vec<User>> {
    User::find_all(pool).await
}
