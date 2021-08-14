use crate::service::user_service;
use actix_web::{get, post, web, Responder, HttpResponse};
use sqlx::MySqlPool;
use crate::common::api::ApiResult;
use captcha::{Captcha, Geometry};
use captcha::filters::{Noise, Wave, Cow};
use crate::model::user::{CaptchaUser, LoginUser, RegisterUser, User, VerifyStatus};
use anyhow::Error;
use crate::AppState;
use r2d2_redis::{redis, redis::ConnectionLike};
use std::ops::{Deref, DerefMut};
use r2d2_redis::redis::{RedisResult, RedisError};
use crate::common::util;
use uuid::Uuid;
use crate::MAILE_RE;

#[get("/user/validate/email/{email}")]
async fn validate_email(web::Path((email)): web::Path<String>, state: AppState) -> impl Responder {
    let legal = MAILE_RE.is_match(&email);
    if !legal {
        return ApiResult::error().data(VerifyStatus::fail()).msg("邮箱格式不合法，请重新出入邮箱");
    } else {
        user_service::check_email_exists(email, &state.get_ref().db_pool).await
    }
}


#[get("/user/validate/username/{username}")]
async fn validate_username(web::Path((username)): web::Path<String>, state: AppState) -> impl Responder {
    user_service::check_username_exists(username, &state.get_ref().db_pool).await
}

#[get("/captcha")]
async fn get_captcha(state: AppState) -> impl Responder {
    let (key, vec) = util::gen_pic_captcha(&mut state.get_ref().redis_pool.get().unwrap()).await;
    HttpResponse::Ok().header("captcha-key", key).body(vec)
}

#[get("/verify/captcha")]
async fn verify_captcha(captcha_user: web::Form<CaptchaUser>, state: AppState) -> impl Responder {
    let connection = &mut state.get_ref().redis_pool.get().unwrap();
    let result = util::redis_get::<String>(&captcha_user.captcha_key, connection).await;
    match result {
        Ok(value) => {
            if value.eq(&captcha_user.captcha_value) {
                let email_verify_code = util::send_email(&captcha_user.email).await;
                util::redis_set(&captcha_user.email, &email_verify_code, 60 * 50, connection).await;
                ApiResult::ok().msg("验证码校验成功，已发送验证码到您邮箱，请查收").data(VerifyStatus::success())
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

#[get("/verify/email")]
async fn verify_email(register_user: web::Form<RegisterUser>, state: AppState) -> impl Responder {
    let connection = &mut state.get_ref().redis_pool.get().unwrap();
    let pool = &state.get_ref().db_pool;
    user_service::register_user(register_user.into_inner(), connection, pool).await
}

#[get("/login")]
async fn login(login_user: web::Form<LoginUser>) -> impl Responder {
    // TODO 登录接口
    ApiResult::ok().msg("").data("")
}

// function that will be called on new Application to configure routes for this module
#[inline]
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_email).
        service(validate_email).
        service(validate_username).
        service(get_captcha).
        service(verify_captcha).
        service(login);
}
