#[macro_use]
extern crate serde;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate r2d2_redis;

use std::env;
use std::time::Duration;

use actix_web::http::ContentEncoding;
use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use r2d2_redis::r2d2::{Pool, PooledConnection};
use r2d2_redis::RedisConnectionManager;
use regex::Regex;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{Error, MySql, MySqlPool};

use crate::common::err::AppError;
use crate::model::user::UserToken;
use chrono::Local;
use common::auth_middleware;
use sqlx::pool::PoolConnection;

mod common;
mod model;
mod repository;
mod route;
mod service;

lazy_static! {
    static ref MAILE_RE: Regex =
        Regex::new(r"^[a-zA-Z0-9_-]+@[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)+$").unwrap();
}

// 在actix_web::web::Data之间共享
#[derive(Clone)]
struct ShareState {
    pub db_pool: MySqlPool,
    pub redis_pool: Pool<RedisConnectionManager>,
}

type AppState = web::Data<ShareState>;
type AppResult<R> = std::result::Result<R, AppError>;

impl ShareState {
    pub async fn get_redis_conn(&self) -> AppResult<PooledConnection<RedisConnectionManager>> {
        self.redis_pool
            .get()
            .map_err(|e| AppError::RedisConnectionError(e))
    }

    pub async fn get_mysql_conn(&self) -> AppResult<PoolConnection<MySql>> {
        let result = self.db_pool.acquire().await;
        result.map_err(|e| AppError::DatabaseError(e))
    }
}

#[actix_web::main]
async fn main() -> AppResult<()> {
    dotenv().ok();
    init_logger();
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = MySqlPoolOptions::new()
        .max_connections(20)
        .min_connections(10)
        .max_lifetime(Duration::from_millis(1800000))
        .idle_timeout(Duration::from_millis(600000))
        .connect(&db_url)
        .await?;

    let manager = RedisConnectionManager::new("redis://localhost").unwrap();

    let redis_pool = r2d2_redis::r2d2::Pool::builder().build(manager).unwrap();

    let state = ShareState {
        db_pool,
        redis_pool,
    };
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middleware::Logger::new("%r %s"))
            .wrap(middleware::Compress::new(ContentEncoding::Br))
            .wrap(auth_middleware::Auth)
            .configure(route::init_all)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;
    Ok(())
}

#[inline]
fn init_logger() {
    use chrono::Local;
    use std::io::Write;

    std::env::set_var("RUST_LOG", "info");
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    // 设置日志打印格式
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                &record.args()
            )
        })
        .init();
    info!("env_logger initialized.");
}
