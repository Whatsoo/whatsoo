use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use chrono::DateTime;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub pk_id: i32,
    pub uk_username: String,
    pub uk_e_mail: String,
    pub avatar: Option<String>,
    pub blog_url: Option<String>,
    pub introduce: Option<String>,
    pub github_uid: Option<String>,
    pub create_time: DateTime<chrono::Utc>,
    pub update_time: DateTime<chrono::Utc>,
}

// implementation of Actix Responder for User struct so we can return User from action handler
impl Responder for User {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    #[inline]
    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

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
                uk_e_mail: rec.uk_e_mail,
                avatar: rec.avatar,
                blog_url: rec.blog_url,
                introduce: rec.introduce,
                github_uid: rec.github_uid,
                create_time: rec.create_time,
                update_time: rec.update_time,
            });
        }

        Ok(users)
    }
}
