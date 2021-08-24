use axum::handler::{get, post};
use axum::routing::BoxRoute;
use axum::Router;

use crate::route::user_route::{
    get_captcha, login, validate_email, validate_username, verify_captcha, verify_email,
};

pub mod user_route;

#[inline]
pub fn init() -> Router<BoxRoute> {
    Router::new()
        .route("/user/validate/email/:email", get(validate_email))
        .route("/user/validate/username/:username", get(validate_username))
        .route("/verify/captcha", get(verify_captcha))
        .route("/verify/email", get(verify_email))
        .route("/login", post(login))
        .route("/captcha", get(get_captcha))
        .boxed()
}
