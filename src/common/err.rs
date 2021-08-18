use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("databaseError: {0}")]
    DatabaseError(#[from] sqlx::Error),
    // #[error("business error")]
    // BusinessError(BusinessErrorType),
    #[error("Error on encode or decode password")]
    PwdHashError(#[from] argon2::password_hash::Error),
    #[error("Error on argon2")]
    Argon2Error(#[from] argon2::Error),
    #[error("jwt error")]
    JWTError(#[from] jsonwebtoken::errors::Error),
    #[error("Error on operate file")]
    IoError(#[from] std::io::Error),
    #[error("Environment variable must be set")]
    EnvironmentVariableNotSet(#[from] std::env::VarError),
    #[error("Actix Error: {0}")]
    ActixError(#[from] actix_web::Error),
    #[error("Redis Connection Error: {0}")]
    RedisConnectionError(#[from] r2d2_redis::r2d2::Error),
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde::de::value::Error),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::ActixError(_) => StatusCode::BAD_REQUEST,
            AppError::SerdeError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::IoError(_) => "IO_ERROR",
            AppError::EnvironmentVariableNotSet(_) => "ENVIRONMENT_VARIABLE_MUST_BE_SET",
            AppError::ActixError(_) => "SERVER_ERROR",
            AppError::RedisConnectionError(_) => "REDIS_CONNECTION_ERROR",
            AppError::SerdeError(_) => "SERDE_ERROR",
        }
    }

    pub fn message(&self) -> String {
        match self {
            AppError::DatabaseError(e) => e.to_string(),
            AppError::IoError(e) => e.to_string(),
            AppError::EnvironmentVariableNotSet(e) => e.to_string(),
            AppError::ActixError(e) => e.to_string(),
            AppError::RedisConnectionError(e) => e.to_string(),
            AppError::SerdeError(e) => e.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponseWrapper {
    code: String,
    msg: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.status()
    }

    fn error_response(&self) -> HttpResponse {
        let wrapper = ErrorResponseWrapper {
            code: self.code().to_string(),
            msg: self.message(),
        };
        if self.status().is_server_error() {
            error!(
                "[status: {}] [code: {}] [message:{}]",
                self.status_code(),
                self.code(),
                self.message()
            );
        }
        HttpResponse::build(self.status_code()).json(wrapper)
    }
}
