use std::convert::Infallible;
use std::fmt::{Display, Formatter};

use axum::body::{Body, Bytes, Full};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use thiserror::Error;

use crate::common::api::ApiResult;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("databaseError: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("business error")]
    BusinessError(i32, &'static str),
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
    // #[error("Hyper error")]
    // HyperError(#[from] hyper::error::Error),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::SerdeError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::IoError(_) => "IO_ERROR",
            AppError::EnvironmentVariableNotSet(_) => "ENVIRONMENT_VARIABLE_MUST_BE_SET",
            AppError::RedisConnectionError(_) => "REDIS_CONNECTION_ERROR",
            AppError::SerdeError(_) => "SERDE_ERROR",
            AppError::RedisGetError(_) => "Redis_GET_ERROR",
            AppError::PwdHashError(_) => "PASSWORD_HASH_ERROR",
            AppError::BusinessError(_, _) => "BUSINESS_ERROR",
            _ => "INTERNAL_SERVER_ERROR",
        }
    }

    pub fn message(&self) -> String {
        match self {
            AppError::DatabaseError(e) => e.to_string(),
            AppError::IoError(e) => e.to_string(),
            AppError::EnvironmentVariableNotSet(e) => e.to_string(),
            AppError::RedisConnectionError(e) => e.to_string(),
            AppError::SerdeError(e) => e.to_string(),
            AppError::RedisGetError(e) => e.to_string(),
            AppError::BusinessError(_, msg) => msg.to_string(),
            _ => String::from("Internal server error"),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponseWrapper {
    code: String,
    msg: String,
}

impl IntoResponse for AppError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let body = Json(json!({
            "code": self.code(),
            "msg": self.message(),
        }));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
