use r2d2::PooledConnection;
use redis::Client;
use sqlx::MySqlPool;

use crate::common::api::ApiResult;
use crate::common::util;
use crate::model::user::{RegisterUser, User, VerifyStatus};
use crate::AppResult;

pub async fn check_email_exists(email: String, pool: &MySqlPool) -> ApiResult<VerifyStatus> {
    let exists = User::count_by_email(email, pool).await;
    match exists {
        Ok(count) => {
            if count == 0 {
                ApiResult::ok().data(VerifyStatus::success()).msg("邮箱验证成功")
            } else {
                ApiResult::error().data(VerifyStatus::fail()).msg("邮箱已注册，请登录")
            }
        }
        Err(e) => {
            error!("邮箱验证出错，报错信息: {}", e.to_string());
            ApiResult::error()
                .data(VerifyStatus::fail())
                .msg("邮箱验证，服务器出错")
        }
    }
}

pub async fn check_username_exists(username: String, pool: &MySqlPool) -> ApiResult<VerifyStatus> {
    let exists = User::count_by_username(username, pool).await;
    match exists {
        Ok(count) => {
            if count == 0 {
                ApiResult::ok().data(VerifyStatus::success()).msg("用户名认证成功")
            } else {
                ApiResult::error()
                    .data(VerifyStatus::fail())
                    .msg("用户名已存在，请登录")
            }
        }
        Err(e) => {
            error!("用户名验证出错，报错信息: {}", e.to_string());
            ApiResult::error()
                .data(VerifyStatus::fail())
                .msg("用户名验证，服务器错误")
        }
    }
}

pub async fn register_user(
    mut register_user: RegisterUser,
    connection: &mut PooledConnection<Client>,
    pool: &MySqlPool,
) -> AppResult<ApiResult<VerifyStatus>> {
    let email = &register_user.uk_email;
    let verify_code = &register_user.email_verify_code;
    let value = util::redis_get::<String>(email, connection).await?;

    if value.eq(verify_code) {
        // 邮箱校验成功即注册成功
        // 加密密码
        register_user.user_password = util::encode_pwd(&register_user.user_password).await?;
        match User::insert_one_user(register_user, pool).await {
            Ok(id) => {
                tracing::info!("用户注册成功，用户id: {}", id);
                Ok(ApiResult::ok()
                    .msg("邮箱验证码校验正确，注册成功")
                    .data(VerifyStatus::success()))
            }
            Err(e) => {
                error!("插入用户失败，失败原因:{}", e.to_string());
                Err(e)
            }
        }
    } else {
        Ok(ApiResult::ok().msg("邮箱验证码校验失败").data(VerifyStatus::fail()))
    }
}
