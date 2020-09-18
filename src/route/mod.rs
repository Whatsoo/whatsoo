pub mod user_route;
use actix_web::web;

#[inline]
pub fn init_all(cfg: &mut web::ServiceConfig) {
    user_route::init(cfg);
    user_route::init(cfg);
}
