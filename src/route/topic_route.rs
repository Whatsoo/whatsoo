use crate::common::api::ApiResult;
use crate::model::user::UserToken;

pub(crate) async fn create_topic(user_token: UserToken) -> ApiResult<UserToken> {
    info!("{:#?}", user_token);
    // todo!("插入主题数据")
    ApiResult::ok().msg("解析成功").data(user_token)
}
