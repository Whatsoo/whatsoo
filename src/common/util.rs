extern crate lettre;
extern crate lettre_email;
extern crate mime;

use std::ops::DerefMut;

use argon2::password_hash::Error;
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier, Version,
};
use captcha::filters::{Cow, Noise, Wave};
use captcha::{Captcha, Geometry};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};
use lettre_email::Email;
use r2d2_redis::r2d2::PooledConnection;
use r2d2_redis::redis::{FromRedisValue, RedisResult};
use r2d2_redis::{redis, RedisConnectionManager};
use rand_core::OsRng;
use uuid::Uuid;

use crate::common::constant::TOKEN_SECRET;
use crate::common::err::AppError;
use crate::model::user::UserToken;
use crate::AppResult;

pub async fn send_email(email_receiver: &str) -> String {
    let mine_email = "nova-me@whatsoo.org";
    let smtp_server = "smtp.exmail.qq.com";
    let password = "Zsl19951210"; //需要生成应用专用密码
    let verify_code = SaltString::generate(&mut OsRng);

    let creds = Credentials::new(mine_email.to_string(), password.to_string());

    // Open connection to Gmail
    let mut mailer = SmtpClient::new_simple(smtp_server)
        .unwrap()
        .credentials(creds)
        .transport();

    let email = Email::builder()
        .to(email_receiver)
        .from(mine_email)
        .subject("whatsoo论坛邮箱验证码")
        .html(format!("<h3>{}</h3>", verify_code.as_str()))
        .build()
        .unwrap();

    // Send the email
    let result = mailer.send(email.into());

    if result.is_ok() {
        info!("Email sent");
    } else {
        info!("Could not send email: {:?}", result);
    }

    info!("{:?}", result);
    mailer.close();
    String::from(verify_code.as_str())
}

pub async fn encode_pwd(pwd: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(None, 3, 1024, 1, Version::V0x13)
        .map_err(|e| AppError::PwdHashError(e.into()))?;

    let result = argon2
        .hash_password_simple(pwd.as_bytes(), salt.as_ref())
        .map_err(|e| AppError::PwdHashError(e))?;
    Ok(result.to_string())
}

pub async fn verify_pwd(input_pwd: &str, db_pwd: &str) -> AppResult<bool> {
    let db_pwd_hash = PasswordHash::new(db_pwd).map_err(|e| AppError::PwdHashError(e))?;
    let is_success = Argon2::new(None, 3, 1024, 1, Version::V0x13)
        .map_err(|e| AppError::PwdHashError(e.into()))?
        .verify_password(input_pwd.as_bytes(), &db_pwd_hash)
        .is_ok();
    Ok(is_success)
}

// expired_time过期时间单位为秒
pub async fn redis_set(
    key: &str,
    value: &str,
    expired_time: i32,
    connection: &mut PooledConnection<RedisConnectionManager>,
) {
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(expired_time)
        .execute(connection.deref_mut());
}

pub async fn redis_get<T: FromRedisValue>(
    key: &str,
    connection: &mut PooledConnection<RedisConnectionManager>,
) -> AppResult<T> {
    redis::cmd("GET")
        .arg(key)
        .query::<T>(connection.deref_mut())
        .map_err(|e| AppError::RedisGetError(e))
}

pub async fn gen_pic_captcha(
    connection: &mut PooledConnection<RedisConnectionManager>,
) -> AppResult<(String, Vec<u8>)> {
    let mut c = Captcha::new();
    c.add_chars(4)
        .apply_filter(Noise::new(0.0))
        .apply_filter(Wave::new(1.5, 10.0))
        .view(220, 120)
        .set_color([255, 245, 238])
        .apply_filter(
            Cow::new()
                .min_radius(40)
                .max_radius(50)
                .circles(1)
                .area(Geometry::new(40, 150, 50, 70)),
        );
    let captcha_value = c.chars_as_string();
    let key = Uuid::new_v4().to_urn().to_string();
    info!("key: {} 图形验证码: {}", &key, &captcha_value);
    redis_set(&key, &captcha_value, 60 * 5, connection).await;
    let vec = c.as_png().unwrap();
    Ok((key, vec))
}

pub async fn validate_captcha(
    key: &str,
    value: &str,
    connection: &mut PooledConnection<RedisConnectionManager>,
) -> bool {
    let result = redis_get::<String>(key, connection).await;
    match result {
        Ok(v) => {
            if v.eq(value) {
                true
            } else {
                false
            }
        }
        Err(e) => {
            error!("获取验证码缓存失败, 失败原因: {}", e.to_string());
            false
        }
    }
}

pub async fn token_encode(user_token: &UserToken) -> String {
    encode(
        &Header::default(),
        user_token,
        &EncodingKey::from_secret(TOKEN_SECRET),
    )
    .map_err(|e| AppError::JWTError(e))
    .ok()
    .unwrap()
}

pub async fn token_decode(user_token: &str) -> UserToken {
    let token = decode::<UserToken>(
        user_token,
        &DecodingKey::from_secret(TOKEN_SECRET),
        &Validation::default(),
    );
    match token {
        Ok(t) => t.claims,
        Err(e) => {
            println!("{}", e.to_string());
            UserToken::default()
        }
    }
}
