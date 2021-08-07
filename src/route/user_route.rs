use crate::service::user_service;
use actix_web::{get, web, Responder, HttpResponse};
use sqlx::MySqlPool;
use crate::common::api::ApiResult;
use captcha::{Captcha, Geometry};
use captcha::filters::{Noise, Wave, Cow};

#[get("/user/all")]
async fn find_all_users(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let res = user_service::find_all(db_pool.get_ref()).await;
    match res {
        Ok(users) => ApiResult::ok().data(users).msg("查询用户成功"),
        Err(e) => {
            error!("查询用户失败，报错信息: {}",e.to_string());
            ApiResult::error().data(Vec::new()).msg("查询用户失败")
        }
    }
}

#[get("/captcha")]
async fn get_captcha() -> impl Responder {
    let mut c = Captcha::new();
    c.add_chars(4)
        .apply_filter(Noise::new(0.0))
        .apply_filter(Wave::new(1.5, 10.0))
        .view(220, 120)
        .set_color([255, 245, 238])
        .apply_filter(
            Cow::new()
                .min_radius(40)
                .max_radius(50)
                .circles(1)
                .area(Geometry::new(40, 150, 50, 70)),
        );
    let a = c.chars_as_string();
    // todo!("验证码存储到redis中，记录key，value，返回key到header中");
    info!("验证码{}", &a);
    let vec = c.as_png().unwrap();
    HttpResponse::Ok().body(vec)
}

// function that will be called on new Application to configure routes for this module
#[inline]
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_users);
    cfg.service(get_captcha);
}
