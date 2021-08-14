use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;
use crate::common::date_format;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub pk_id: u64,
    pub uk_username: String,
    pub uk_email: String,
    pub user_password: String,
    pub salt: String,
    pub avatar: Option<String>,
    pub blog_url: Option<String>,
    pub introduce: Option<String>,
    pub github_uid: Option<String>,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
    #[serde(with = "date_format")]
    pub update_time: NaiveDateTime,
    #[serde(with = "date_format")]
    pub last_login_time: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUser {
    pub uk_username: String,
    pub uk_email: String,
    pub email_verify_code: String,
    pub user_password: String,
}

#[derive(Debug, Deserialize)]
pub struct CaptchaUser {
    pub captcha_key: String,
    pub captcha_value: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub captcha_key: String,
    pub captcha_value: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct VerifyStatus {
    is_success: bool,
}

impl VerifyStatus {
    pub fn success() -> Self {
        Self {
            is_success: true
        }
    }

    pub fn fail() -> Self {
        Self {
            is_success: false
        }
    }
}