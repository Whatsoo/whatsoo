use sqlx::MySqlPool;

use crate::common::err::AppError;
use crate::model::user::{RegisterUser, User};
use crate::AppResult;

impl User {
    pub async fn count_by_email(email: String, pool: &MySqlPool) -> AppResult<i64> {
        sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM user
            WHERE uk_email = ?
            "#,
            email
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e))
        .map(|res| res.count)
    }

    pub async fn count_by_username(username: String, pool: &MySqlPool) -> AppResult<i64> {
        sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM user
            WHERE uk_username = ?
            "#,
            username
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e))
        .map(|res| res.count)
    }

    pub async fn insert_one_user(user: RegisterUser, pool: &MySqlPool) -> AppResult<u64> {
        sqlx::query!(
            r#"
            INSERT INTO user
                (uk_username, uk_email, user_password)
            VALUES
                (?, ?, ?);
            "#,
            user.uk_username,
            user.uk_email,
            user.user_password,
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e))
        .map(|done| done.last_insert_id())
    }

    pub async fn find_user_by_email(email: &str, pool: &MySqlPool) -> Option<User> {
        sqlx::query_as!(
            User,
            r#"
            SELECT *
            FROM user
            WHERE uk_email = ?
            "#,
            email
        )
        .fetch_one(pool)
        .await
        .ok()
    }
}
