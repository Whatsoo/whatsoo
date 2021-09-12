use sqlx::MySqlPool;

use crate::{common::err::AppError, model::topic::TopicFront, AppResult};

#[inline]
pub async fn insert_one_topic(new_topic: TopicFront, pool: &MySqlPool) -> AppResult<bool> {
    let user_id = new_topic.user_id.ok_or(AppError::BusinessError(500, "用户不能为空"))?;
    sqlx::query!(
        r#"
        INSERT INTO topic
            (user_id, title, content, tags, create_user, update_user)
        VALUES
            (?,?,?,?,?,?)
        "#,
        user_id,
        new_topic.title,
        new_topic.content,
        new_topic.tags,
        user_id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e))
    .map(|done| done.last_insert_id() > 0)
}
