use crate::model::user::User;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/user/all")]
async fn find_all_users(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let res = User::find_all(db_pool.get_ref()).await;
    match res {
        Ok(users) => HttpResponse::Ok().json(users),
        _ => HttpResponse::BadRequest().body("Error trying to read all todos from database"),
    }
}

// function that will be called on new Application to configure routes for this module
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_users);
}
