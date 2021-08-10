use crate::model::user::{User, RegisterUser};
use anyhow::Result;
use sqlx::MySqlPool;

impl User {
    pub async fn count_by_email(email: String, pool: &MySqlPool) -> Result<i64> {
        let rec = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM user
            WHERE uk_email = ?
            "#,
            email
        )
            .fetch_one(pool)
            .await?;
        info!("{:#?}",rec);
        Ok(rec.count)
    }

    pub async fn count_by_username(username: String, pool: &MySqlPool) -> Result<i64> {
        let rec = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM user
            WHERE uk_username = ?
            "#,
            username
        )
            .fetch_one(pool)
            .await?;
        info!("{:#?}",rec);
        Ok(rec.count)
    }

    pub async fn insert_one_user(user: RegisterUser, pool: &MySqlPool) -> Result<i64> {
        let rec = sqlx::query!(
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
            .await?;
        info!("插入用户返回值{:#?}",rec);
        // TODO 正确返回值
        Ok(1)
    }
}
