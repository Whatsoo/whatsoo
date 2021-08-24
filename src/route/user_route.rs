use std::borrow::BorrowMut;

use axum::extract::{Extension, Form, Path};
use axum::handler::{get, post};
use axum::http::header::HeaderName;
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use axum::response::{Headers, IntoResponse};
use axum::routing::BoxRoute;
use axum::{extract, Router};
use chrono::Local;

use crate::common::api::ApiResult;
use crate::common::constant::TOKEN_HEADER_NAME;
use crate::common::err::AppError;
use crate::common::util;
use crate::model::user::{CaptchaUser, LoginUser, RegisterUser, User, UserToken, VerifyStatus};
use crate::service::user_service;
use crate::MAILE_RE;
use crate::{AppResult, ShareState};

pub(crate) async fn validate_email(
    Path(email): Path<String>,
    state: Extension<ShareState>,
) -> AppResult<ApiResult<VerifyStatus>> {
    let legal = MAILE_RE.is_match(&email);
    if !legal {
        return ApiResult::error()
            .data(VerifyStatus::fail())
            .msg("邮箱格式不合法，请重新出入邮箱")
            .into();
    } else {
        user_service::check_email_exists(email, &state.db_pool)
            .await
            .into()
    }
}

pub(crate) async fn validate_username(
    Path(username): Path<String>,
    state: Extension<ShareState>,
) -> ApiResult<VerifyStatus> {
    user_service::check_username_exists(username, &state.db_pool).await
}

pub(crate) async fn get_captcha(
    state: Extension<ShareState>,
) -> AppResult<(StatusCode, HeaderMap, Vec<u8>)> {
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
    let connection = &mut state.get_redis_conn().await?;
    let is_valid = util::validate_captcha(
        &captcha_user.captcha_key,
        &captcha_user.captcha_value,
        connection,
    )
    .await;
    if is_valid {
        let legal = MAILE_RE.is_match(&captcha_user.email);
        if legal {
            let email_verify_code = util::send_email(&captcha_user.email).await;
            util::redis_set(&captcha_user.email, &email_verify_code, 60 * 50, connection).await;
            return ApiResult::ok()
                .msg("验证码校验成功，已发送验证码到您邮箱，请查收")
                .data(VerifyStatus::success())
                .into();
        } else {
            return ApiResult::ok()
                .msg("验证码校验失败, 邮箱格式不合法")
                .data(VerifyStatus::fail())
                .into();
        }
    } else {
        ApiResult::ok()
            .msg("验证码校验失败")
            .data(VerifyStatus::fail())
            .into()
    }
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
) -> AppResult<ApiResult<VerifyStatus>> {
    let connection = &mut state.get_redis_conn().await?;
    let pool = &state.db_pool;
    let is_valid = util::validate_captcha(
        &login_user.captcha_key,
        &login_user.captcha_value,
        connection,
    )
    .await;
    // 验证码不正确直接返回
    if !is_valid {
        return ApiResult::ok()
            .msg("验证码校验失败")
            .data(VerifyStatus::fail())
            .into();
    }
    let user = User::find_user_by_email(&login_user.email, pool).await;
    return if let Some(u) = user {
        let login_success = util::verify_pwd(&login_user.password, &u.user_password).await?;
        if login_success {
            let exp: usize = if login_user.forever {
                (Local::now().timestamp() + 60 * 60 * 24 * 365) as usize
            } else {
                (Local::now().timestamp() + 60 * 60 * 24 * 7) as usize
            };
            let user_token =
                util::token_encode(&UserToken::new(u.pk_id, u.uk_username, u.uk_email, exp))
                    .await;
            ApiResult::ok()
                .msg("登录成功")
                .data(VerifyStatus::success())
                .into()
        } else {
            ApiResult::error()
                .msg("登录失败，用户名或密码错误")
                .data(VerifyStatus::fail())
                .into()
        }
    } else {
        ApiResult::error()
            .msg("登录失败，用户名或密码错误")
            .data(VerifyStatus::fail())
            .into()
    };
}
