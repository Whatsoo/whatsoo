#[macro_use]
extern crate serde;
#[macro_use]
extern crate log;

use actix_web::{middleware, App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::time::Duration;
use actix_web::http::ContentEncoding;
use argon2::Version;

mod common;
mod model;
mod route;
mod service;
mod repository;
mod auth_middleware;

#[actix_web::main]
async fn main() -> Result<()> {
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

    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
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
