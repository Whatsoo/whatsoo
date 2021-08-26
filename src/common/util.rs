extern crate lettre;
extern crate lettre_email;
extern crate mime;

use std::ops::DerefMut;

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier, Version,
};
use captcha::filters::{Cow, Noise, Wave};
use captcha::{Captcha, Geometry};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};
use lettre_email::Email;
use r2d2::PooledConnection;
use rand_core::OsRng;
use redis::{Client, FromRedisValue};
use tokio::sync::MutexGuard;
use uuid::Uuid;

use crate::common::constant::TOKEN_SECRET;
use crate::common::err::AppError;
use crate::model::user::UserToken;
use crate::AppResult;

use self::lettre::SmtpTransport;

pub async fn send_email(email_receiver: &str, verify_code: &str, mut smtp_transport: MutexGuard<'_, SmtpTransport>) {
    let mine_email = "nova-me@whatsoo.org";
    let smtp_server = "smtp.exmail.qq.com";
    let password = "Zsl19951210"; //需要生成应用专用密码

    let email = Email::builder()
        .to(email_receiver)
        .from(mine_email)
        .subject("whatsoo论坛邮箱验证码")
        .html(format!("<h3>{}</h3>", verify_code))
        .build()
        .unwrap();

    // Send the email
    let result = smtp_transport.send(email.into());

    if result.is_ok() {
        tracing::info!("Email sent");
    } else {
        info!("Could not send email: {:?}", result);
    }
    info!("{:?}", result);
}

pub async fn encode_pwd(pwd: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(None, 3, 1024, 1, Version::V0x13).map_err(|e| AppError::PwdHashError(e.into()))?;

    let result = argon2.hash_password_simple(pwd.as_bytes(), salt.as_ref())?;
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
pub async fn redis_set(key: &str, value: &str, expired_time: i32, connection: &mut PooledConnection<Client>) {
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(expired_time)
        .execute(connection.deref_mut());
}

pub async fn redis_get<T: FromRedisValue>(key: &str, connection: &mut PooledConnection<Client>) -> AppResult<T> {
    Ok(redis::cmd("GET").arg(key).query::<T>(connection.deref_mut())?)
}

pub async fn gen_pic_captcha() -> AppResult<(String, String, Vec<u8>)> {
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
    let vec = c.as_png().unwrap();
    Ok((key, captcha_value, vec))
}

pub async fn validate_captcha(key: &str, value: &str, connection: &mut PooledConnection<Client>) -> bool {
    let result = redis_get::<String>(key, connection).await;
    match result {
        Ok(v) => v.eq(value),
        Err(_) => false,
    }
}

pub async fn token_encode(user_token: &UserToken) -> AppResult<String> {
    let option = encode(&Header::default(), user_token, &EncodingKey::from_secret(TOKEN_SECRET))
        .map_err(AppError::JWTError)
        .ok();
    match option {
        None => Err(AppError::BusinessError(500, "token解密失败")),
        Some(s) => Ok(s),
    }
}

pub async fn token_decode(user_token: &str) -> Option<UserToken> {
    let token = decode::<UserToken>(
        user_token,
        &DecodingKey::from_secret(TOKEN_SECRET),
        &Validation::default(),
    );
    match token {
        Ok(t) => Some(t.claims),
        Err(e) => {
            error!("{}", e.to_string());
            None
        }
    }
}
