use axum::extract::Extension;
use axum::Json;

use crate::common::api::ApiResult;
use crate::common::err::AppError;
use crate::model::topic::TopicFront;
use crate::model::user::{UserToken, VerifyStatus};
use crate::repository::topic_repository;
use crate::{AppResult, ShareState};

pub(crate) async fn create_topic(
    Json(mut new_topic): Json<TopicFront>,
    user_token: UserToken,
    state: Extension<ShareState>,
) -> AppResult<ApiResult<VerifyStatus>> {
    new_topic.user_id = Some(user_token.user_id);
    if topic_repository::insert_one_topic(new_topic, &state.db_pool).await? {
        Ok(ApiResult::ok().msg("创建主题成功").data(VerifyStatus::success()))
    } else {
        Err(AppError::BusinessError(500, "创建主题失败"))
    }
}
