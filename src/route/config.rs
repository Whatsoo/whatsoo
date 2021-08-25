use axum::handler::{get, post};
use axum::routing::BoxRoute;
use axum::Router;

use crate::route::topic_route::create_topic;
use crate::route::user_route::{get_captcha, login, validate_email, validate_username, verify_captcha, verify_email};

#[inline]
pub fn init() -> Router<BoxRoute> {
    Router::new()
        .route("/user/validate/email/:email", get(validate_email))
        .route("/user/validate/username/:username", get(validate_username))
        .route("/verify/captcha", post(verify_captcha))
        .route("/verify/email", post(verify_email))
        .route("/login", post(login))
        .route("/captcha", get(get_captcha))
        .route("/topic", get(create_topic))
        .boxed()
}
