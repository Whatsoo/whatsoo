use crate::model::user::User;
use anyhow::Result;
use sqlx::MySqlPool;

impl User {
    pub async fn find_all(pool: &MySqlPool) -> Result<Vec<User>> {
        let mut users = vec![];
        let recs = sqlx::query!(
            r#"
            SELECT *
            FROM user
            ORDER BY pk_id
            "#
        )
            .fetch_all(pool)
            .await?;

        for rec in recs {
            users.push(User {
                pk_id: rec.pk_id,
                uk_username: rec.uk_username,
                uk_email: rec.uk_email,
                user_password: rec.user_password,
                salt: rec.salt,
                avatar: rec.avatar,
                blog_url: rec.blog_url,
                introduce: rec.introduce,
                github_uid: rec.github_uid,
                create_time: rec.create_time,
                update_time: rec.update_time,
                last_login_time: rec.last_login_time,
            });
        }

        Ok(users)
    }
}
