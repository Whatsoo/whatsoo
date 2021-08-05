use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::future::{ready, Ready};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Topic {
    pub pk_id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub tags: String,
    pub like_times: i32,
    pub click_times: i32,
    pub create_time: NaiveDateTime,
    pub create_user: i64,
    pub update_time: NaiveDateTime,
    pub update_user: i64,
}

// implementation of Actix Responder for User struct so we can return User from action handler
impl Responder for Topic {
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
