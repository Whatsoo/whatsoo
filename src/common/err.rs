use std::convert::Infallible;

use axum::body::{Bytes, Full};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("databaseError: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Error on encode or decode password")]
    PwdHashError(#[from] argon2::password_hash::Error),
    #[error("jwt error")]
    JWTError(#[from] jsonwebtoken::errors::Error),
    #[error("Error on operate file")]
    IoError(#[from] std::io::Error),
    #[error("Environment variable must be set")]
    EnvironmentVariableNotSet(#[from] std::env::VarError),
    #[error("Redis Connection Error: {0}")]
    RedisConnectionError(#[from] r2d2::Error),
    #[error("Redis error")]
    RedisGetError(#[from] redis::RedisError),
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde::de::value::Error),
    #[error("Axum http error")]
    AxumHttpError(#[from] axum::http::Error),
    #[error("Serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("business error")]
    BusinessError(i32, &'static str),
}

impl AppError {
    pub fn message(&self) -> String {
        match self {
            AppError::BusinessError(_, msg) => msg.to_string(),
            _ => String::from("Internal server error"),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponseWrapper {
    code: i32,
    msg: String,
}

impl IntoResponse for AppError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let body = Json(json!({
            "code": 500,
            "msg": self.message(),
        }));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
