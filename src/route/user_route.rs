use crate::common::api::ApiResult;
use crate::common::util;
use crate::model::user::{CaptchaUser, LoginUser, RegisterUser, VerifyStatus};
use crate::service::user_service;
use crate::AppState;
use crate::MAILE_RE;
use actix_web::{get, post, web, HttpResponse, Responder};
use r2d2_redis::r2d2::PooledConnection;
use r2d2_redis::redis::RedisError;
use r2d2_redis::RedisConnectionManager;

#[get("/user/validate/email/{email}")]
async fn validate_email(web::Path(email): web::Path<String>, state: AppState) -> impl Responder {
    let legal = MAILE_RE.is_match(&email);
    if !legal {
        return ApiResult::error()
            .data(VerifyStatus::fail())
            .msg("邮箱格式不合法，请重新出入邮箱");
    } else {
        user_service::check_email_exists(email, &state.get_ref().db_pool).await
    }
}

#[get("/user/validate/username/{username}")]
async fn validate_username(
    web::Path(username): web::Path<String>,
    state: AppState,
) -> impl Responder {
    user_service::check_username_exists(username, &state.get_ref().db_pool).await
}

#[get("/captcha")]
async fn get_captcha(state: AppState) -> impl Responder {
    let (key, vec) = util::gen_pic_captcha(&mut state.get_ref().redis_pool.get().unwrap()).await;
    HttpResponse::Ok().header("captcha-key", key).body(vec)
}

#[post("/verify/captcha")]
async fn verify_captcha(captcha_user: web::Form<CaptchaUser>, state: AppState) -> impl Responder {
    let connection = &mut state.get_ref().redis_pool.get().unwrap();
    let is_valid = util::validate_captcha(
        &captcha_user.captcha_key,
        &captcha_user.captcha_value,
        connection,
    )
    .await;
    if is_valid {
        ApiResult::ok()
            .msg("验证码校验成功，已发送验证码到您邮箱，请查收")
            .data(VerifyStatus::success())
    } else {
        ApiResult::ok()
            .msg("验证码校验失败")
            .data(VerifyStatus::fail())
    }
}

#[post("/verify/email")]
async fn verify_email(register_user: web::Form<RegisterUser>, state: AppState) -> impl Responder {
    let connection = &mut state.get_ref().redis_pool.get().unwrap();
    let pool = &state.get_ref().db_pool;
    user_service::register_user(register_user.into_inner(), connection, pool).await
}

#[get("/login")]
async fn login(login_user: web::Form<LoginUser>, state: AppState) -> impl Responder {
    let connection = &mut state.get_ref().redis_pool.get().unwrap();
    // TODO 验证码是否正确
    let is_valid = util::validate_captcha(
        &login_user.captcha_key,
        &login_user.captcha_value,
        connection,
    )
    .await;
    if is_valid {
    } else {
        return ApiResult::ok()
            .msg("验证码校验失败")
            .data(VerifyStatus::fail());
    }
    // TODO 根据email查询用户是否存在
    // TODO 密码是否正确
    // TODO 登录成功返回token
    ApiResult::ok().msg("").data(VerifyStatus::success())
}

// function that will be called on new Application to configure routes for this module
#[inline]
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_email)
        .service(validate_email)
        .service(validate_username)
        .service(get_captcha)
        .service(verify_captcha)
        .service(login);
}
