#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate tracing;

use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::Request;
use axum::AddExtensionLayer;
use dotenv::dotenv;
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, SmtpTransport};
use r2d2::{Pool, PooledConnection};
use redis::Client;
use regex::Regex;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::pool::PoolConnection;
use sqlx::{MySql, MySqlPool};
use tokio::sync::Mutex;
use tower::filter::AsyncFilterLayer;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

use crate::common::err::AppError;
use crate::route::config;

mod common;
mod model;
mod repository;
mod route;
mod service;

lazy_static! {
    static ref MAILE_RE: Regex = Regex::new(r"^[a-zA-Z0-9_-]+@[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)+$").unwrap();
    static ref USERNAME_RE: Regex = Regex::new(r"^[a-zA-Z0-9](5,10)$").unwrap();
}

#[derive(Clone)]
struct ShareState {
    pub db_pool: MySqlPool,
    pub redis_pool: Pool<Client>,
    pub smtp_transport: Arc<Mutex<SmtpTransport>>,
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
    let db_url = env::var("DATABASE_URL").expect("数据库连接未设置");
    let db_pool = MySqlPoolOptions::new()
        .max_connections(20)
        .min_connections(10)
        .max_lifetime(Duration::from_millis(1800000))
        .idle_timeout(Duration::from_millis(600000))
        .connect(&db_url)
        .await?;
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let redis_pool = r2d2::Pool::builder().max_size(20).min_idle(Some(10)).build(client)?;

    let mine_email = &env::var("MINE_EMAIL").expect("邮箱账户未设置");
    let smtp_server = &env::var("SMTP_SERVER").expect("邮箱服务器未设置");
    let password = &env::var("PASSWORD").expect("邮箱服务器密码未设置");
    let credentials = Credentials::new(mine_email.to_string(), password.to_string());

    // Open connection to QQMail
    let mailer = SmtpClient::new_simple(smtp_server)
        .unwrap()
        .credentials(credentials)
        .transport();
    let smtp_transport = Arc::new(Mutex::new(mailer));
    let state = ShareState {
        db_pool,
        redis_pool,
        smtp_transport,
    };

    let middleware_stack = ServiceBuilder::new()
        .timeout(Duration::from_secs(30))
        .load_shed()
        .concurrency_limit(10000)
        .layer(CompressionLayer::new().br(true))
        .layer(AsyncFilterLayer::new(filter))
        .layer(AddExtensionLayer::new(state))
        .into_inner();

    let app = config::init().layer(middleware_stack);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
    Ok(())
}

async fn filter(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    tracing::info!("path is : {} [{}]", req.method(), req.uri().path());
    todo!("添加过滤条件，检查Token是否过期");
    Ok(req)
}
