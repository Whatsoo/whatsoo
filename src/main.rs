#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use axum::body::Body;
use axum::http::Request;
use axum::service::get;
use axum::{AddExtensionLayer, Router};
use chrono::Local;
use dotenv::dotenv;
use r2d2::{Pool, PooledConnection};
use redis::Client;
use regex::Regex;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::pool::PoolConnection;
use sqlx::{Error, MySql, MySqlPool};
use tower::filter::AsyncFilterLayer;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

use common::auth_middleware;

use crate::common::err::AppError;
use crate::model::user::UserToken;
use crate::route::user_route;

mod common;
mod model;
mod repository;
mod route;
mod service;

lazy_static! {
    static ref MAILE_RE: Regex =
        Regex::new(r"^[a-zA-Z0-9_-]+@[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)+$").unwrap();
    static ref USERNAME_RE: Regex = Regex::new(r"^[a-zA-Z0-9](5,10)$").unwrap();
}

#[derive(Clone)]
struct ShareState {
    pub db_pool: MySqlPool,
    pub redis_pool: Pool<Client>,
}

type AppResult<R> = std::result::Result<R, AppError>;

impl ShareState {
    pub async fn get_redis_conn(&self) -> AppResult<PooledConnection<Client>> {
        Ok(self.redis_pool.get()?)
    }

    pub async fn get_mysql_conn(&self) -> AppResult<PoolConnection<MySql>> {
        Ok(self.db_pool.acquire().await?)
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "whatsoo=info")
    }
    tracing_subscriber::fmt::init();
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

    let client = redis::Client::open("redis://127.0.0.1/")?;
    let redis_pool = r2d2::Pool::builder()
        .max_size(20)
        .min_idle(Some(10))
        .build(client)?;
    let state = ShareState {
        db_pool,
        redis_pool,
    };

    let middleware_stack = ServiceBuilder::new()
        // Return an error after 30 seconds
        .timeout(Duration::from_secs(30))
        // Shed load if we're receiving too many requests
        .load_shed()
        // Process at most 100 requests concurrently
        .concurrency_limit(1000)
        // Compress response bodies
        .layer(CompressionLayer::new().br(true))
        .layer(AsyncFilterLayer::new(map_request))
        .layer(AddExtensionLayer::new(state))
        .into_inner();

    let app = route::init().layer(middleware_stack);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn map_request(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    tracing::info!("path is : {} [{}]", req.method(), req.uri().path());
    // todo!("添加过滤条件，检查Token是否过期");
    Ok(req)
}
