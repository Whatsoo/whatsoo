use crate::service::user_service;
use actix_web::{get, web, Responder, HttpResponse};
use sqlx::MySqlPool;
use crate::common::api::ApiResult;
use captcha::{Captcha, Geometry};
use captcha::filters::{Noise, Wave, Cow};
use crate::model::user::{VerifyStatus, User};
use anyhow::Error;
use crate::AppState;
use r2d2_redis::{redis, redis::ConnectionLike};
use std::ops::{Deref, DerefMut};
use crate::MAILE_RE;
use r2d2_redis::redis::{RedisResult, RedisError};
use crate::common::util;
use uuid::Uuid;

#[get("/user/verify/username/{username}")]
async fn verify_username(web::Path((username)): web::Path<String>, state: AppState) -> impl Responder {
    let exists = user_service::check_username_exists(username, &state.get_ref().db_pool).await;
    match exists {
        Ok(count) => {
            if count == 0 {
                ApiResult::ok().data(VerifyStatus::success()).msg("用户名证成功")
            } else {
                ApiResult::error().data(VerifyStatus::fail()).msg("用户名已存在，请登录")
            }
        }
        Err(e) => {
            error!("用户名验证出错，报错信息: {}",e.to_string());
            ApiResult::error().data(VerifyStatus::fail()).msg("用户名验证，服务器出错")
        }
    }
}

#[get("/user/verify/email/{email}")]
async fn verify_email(web::Path((email)): web::Path<String>, state: AppState) -> impl Responder {
    let legal = MAILE_RE.is_match(&email);
    if !legal {
        return ApiResult::error().data(VerifyStatus::fail()).msg("邮箱格式不合法，请重新出入邮箱");
    }
    let exists = user_service::check_email_exists(email, &state.get_ref().db_pool).await;
    match exists {
        Ok(count) => {
            if count == 0 {
                ApiResult::ok().data(VerifyStatus::success()).msg("邮箱验证成功")
            } else {
                ApiResult::error().data(VerifyStatus::fail()).msg("邮箱已注册，请登录")
            }
        }
        Err(e) => {
            error!("邮箱验证出错，报错信息: {}",e.to_string());
            ApiResult::error().data(VerifyStatus::fail()).msg("邮箱验证，服务器出错")
        }
    }
}

#[get("/captcha")]
async fn get_captcha(state: AppState) -> impl Responder {
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
    let connection = &mut state.get_ref().redis_pool.get().unwrap();
    let key = Uuid::new_v4().to_urn().to_string();
    info!("key: {} 验证码: {}",&key, &captcha_value);
    util::redis_set(&key, &captcha_value, 60, connection);
    let vec = c.as_png().unwrap();
    HttpResponse::Ok().header("captcha-key", key).body(vec)
}

#[get("/verify/captcha/{captcha_key}/{captcha}")]
async fn verify_captcha(web::Path((captcha_key, captcha)): web::Path<(String, String)>, state: AppState) -> impl Responder {
    let connection = &mut state.get_ref().redis_pool.get().unwrap();
    let result = util::redis_get::<String>(&captcha_key, connection);
    match result {
        Ok(value) => {
            if value.eq(&captcha) {
                ApiResult::ok().msg("验证码校验成功").data(VerifyStatus::success())
            } else {
                ApiResult::ok().msg("验证码校验失败").data(VerifyStatus::fail())
            }
        }
        Err(e) => {
            error!("获取验证码缓存失败, 失败原因: {}",e.to_string());
            ApiResult::error().msg("验证码已过期，请刷新验证码").data(VerifyStatus::fail())
        }
    }
}

// function that will be called on new Application to configure routes for this module
#[inline]
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_email);
    cfg.service(verify_username);
    cfg.service(get_captcha);
    cfg.service(verify_captcha);
}
