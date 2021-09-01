use std::sync::Arc;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use axum::extract::{Extension, Form, Path};
use axum::http::header::HeaderName;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use chrono::Local;

use crate::common::api::ApiResult;
use crate::common::constant::TOKEN_HEADER_NAME;
use crate::common::err::AppError;
use crate::common::util;
use crate::model::user::{CaptchaUser, FindUserPwd, LoginUser, RegisterUser, User, UserToken, VerifyStatus};
use crate::service::user_service;
use crate::{AppResult, ShareState};

pub(crate) async fn validate_email(
    Path(email): Path<String>,
    state: Extension<ShareState>,
) -> AppResult<ApiResult<VerifyStatus>> {
    util::validate_email(&email).await?;
    user_service::check_email_exists(email, &state.db_pool).await.into()
}

pub(crate) async fn validate_username(
    Path(username): Path<String>,
    state: Extension<ShareState>,
) -> ApiResult<VerifyStatus> {
    user_service::check_username_exists(username, &state.db_pool).await
}

pub(crate) async fn get_captcha(state: Extension<ShareState>) -> AppResult<(StatusCode, HeaderMap, Vec<u8>)> {
    let conn = &mut state.get_redis_conn().await?;
    let (key, captcha_value, vec) = util::gen_pic_captcha().await?;
    util::redis_set(&key, &captcha_value, 60 * 5, conn).await;
    let mut headers = HeaderMap::with_capacity(1usize);
    headers.insert(
        HeaderName::from_static("captcha-key"),
        HeaderValue::from_static(Box::leak(key.into_boxed_str())),
    );
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("image/png"),
    );
    Ok((StatusCode::OK, headers, vec))
}

pub(crate) async fn verify_captcha(
    captcha_user: Form<CaptchaUser>,
    state: Extension<ShareState>,
) -> AppResult<ApiResult<VerifyStatus>> {
    util::validate_email(&captcha_user.email).await?;
    let connection = &mut state.get_redis_conn().await?;
    util::verify_captcha(&captcha_user.captcha_key, &captcha_user.captcha_value, connection).await?;
    // todo!("删除验证码缓存")
    let arc = Arc::clone(&state.smtp_transport);
    let verify_code = String::from(SaltString::generate(&mut OsRng).as_str());
    util::redis_set(&captcha_user.email, &verify_code, 60 * 50, connection).await;
    tokio::spawn(async move {
        let smtp_transport = arc.lock().await;
        util::send_email(&captcha_user.email, &verify_code, smtp_transport).await;
    });
    ApiResult::ok()
        .msg("验证码校验成功，已发送验证码到您邮箱，请查收")
        .data(VerifyStatus::success())
        .into()
}

pub(crate) async fn verify_email(
    register_user: Form<RegisterUser>,
    state: Extension<ShareState>,
) -> AppResult<ApiResult<VerifyStatus>> {
    let connection = &mut state.get_redis_conn().await?;
    let pool = &state.db_pool;
    user_service::register_user(register_user.0, connection, pool).await
}

pub(crate) async fn login(
    login_user: Form<LoginUser>,
    state: Extension<ShareState>,
) -> AppResult<(StatusCode, HeaderMap, Vec<u8>)> {
    let connection = &mut state.get_redis_conn().await?;
    let pool = &state.db_pool;
    util::verify_captcha(&login_user.captcha_key, &login_user.captcha_value, connection).await?;
    let u = User::find_user_by_email(&login_user.email, pool).await?;
    let login_success = util::verify_pwd(&login_user.password, &u.user_password).await?;
    if login_success {
        let exp: usize = if login_user.forever {
            (Local::now().timestamp() + 60 * 60 * 24 * 365) as usize
        } else {
            (Local::now().timestamp() + 60 * 60 * 24 * 7) as usize
        };
        let user_token = util::token_encode(&UserToken::new(u.pk_id, u.uk_username, u.uk_email, exp)).await?;
        let mut headers = HeaderMap::with_capacity(1usize);
        headers.insert(
            HeaderName::from_static(TOKEN_HEADER_NAME),
            HeaderValue::from_static(Box::leak(user_token.into_boxed_str())),
        );
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        let body = serde_json::to_vec(&ApiResult::ok().msg("登录成功").data(VerifyStatus::success()))?;
        Ok((StatusCode::OK, headers, body))
    } else {
        let body = serde_json::to_vec(
            &ApiResult::error()
                .msg("登录失败，用户名或密码错误")
                .data(VerifyStatus::fail()),
        )?;
        let mut headers = HeaderMap::with_capacity(1usize);
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        Ok((StatusCode::OK, headers, body))
    }
}

pub(crate) async fn find_user_pwd(
    Form(find_user_pwd): Form<FindUserPwd>,
    state: Extension<ShareState>,
) -> AppResult<ApiResult<()>> {
    let conn = &mut state.get_redis_conn().await?;
    let pool = &state.db_pool;
    let user = User::find_user_by_email(&find_user_pwd.email, pool).await?;
    match find_user_pwd.email_verify_code {
        None => {
            util::validate_email(&find_user_pwd.email).await?;
            util::verify_captcha(&find_user_pwd.captcha_key, &find_user_pwd.captcha_value, conn).await?;
            let arc = Arc::clone(&state.smtp_transport);
            let verify_code = String::from(SaltString::generate(&mut OsRng).as_str());
            util::redis_set(&find_user_pwd.email, &verify_code, 60 * 50, conn).await;
            tokio::spawn(async move {
                let smtp_transport = arc.lock().await;
                util::send_email(&find_user_pwd.email, &verify_code, smtp_transport).await;
            });
            Ok(ApiResult::ok().msg("验证码已发送至邮箱").data(()))
        }
        Some(code) => {
            let cache_verify_code = util::redis_get::<String>(&find_user_pwd.email, conn).await?;
            if code.eq(&cache_verify_code) {
                let encode_pwd = util::encode_pwd(&find_user_pwd.password).await?;
                let rows_affected = User::update_user_pwd(encode_pwd, user.pk_id, pool).await?;
                if rows_affected == 1 {
                    Ok(ApiResult::ok().msg("修改密码成功").data(()))
                } else {
                    Err(AppError::BusinessError(500, "修改密码失败"))
                }
            } else {
                Err(AppError::BusinessError(500, "邮箱验证码校验错误"))
            }
        }
    }
}

pub(crate) async fn change_user_pwd(
    user_token: UserToken,
    Path((pwd)): Path<String>,
    state: Extension<ShareState>,
) -> AppResult<ApiResult<()>> {
    let pool = &state.db_pool;
    let encode_pwd = util::encode_pwd(&pwd).await?;
    let rows_affected = User::update_user_pwd(encode_pwd, user_token.user_id, pool).await?;
    if rows_affected == 1 {
        Ok(ApiResult::ok().msg("修改密码成功").data(()))
    } else {
        Err(AppError::BusinessError(500, "修改密码失败"))
    }
}
