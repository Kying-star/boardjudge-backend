pub mod judger;
pub mod sys;
pub mod utils;
pub mod web;

use crate::judger::Judger;
use anyhow::{Context, Result};
use axum::Server;
use sea_orm::Database;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::ptr::null_mut;
use std::sync::atomic::AtomicPtr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub web: ConfigWeb,
    pub db: ConfigDb,
    pub judger: ConfigJudger,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigWeb {
    pub root: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigDb {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigJudger {
    pub root: String,
    pub stack_limit: u64,
    pub output_limit: u64,
    pub compiler_c: String,
    pub compiler_cxx: String,
}

static CONFIG: AtomicPtr<Config> = AtomicPtr::new(null_mut());

pub fn config() -> &'static Config {
    use std::sync::atomic::Ordering;
    let ptr = CONFIG.load(Ordering::SeqCst);
    if ptr.is_null() {
        panic!("boot without initializing CONFIG");
    }
    unsafe { &(*ptr) }
}

pub async fn main(p: Config) -> Result<()> {
    use std::sync::atomic::Ordering;
    let p = Box::into_raw(Box::new(p));
    CONFIG
        .compare_exchange(null_mut(), p, Ordering::SeqCst, Ordering::SeqCst)
        .expect("boot twice");
    let url = format!(
        "mysql://{}:{}@{}/{}",
        config().db.username,
        config().db.password,
        config().db.host,
        config().db.database
    );
    let conn = Database::connect(url)
        .await
        .context("database connection failed")?;
    let judger = Judger::daemon();
    let web = tokio::spawn({
        async move {
            let router = self::web::router(conn, judger);
            let addr = SocketAddr::new(
                config()
                    .web
                    .host
                    .parse()
                    .context("failed to parse web.host")?,
                config().web.port,
            );
            tracing::info!("listening on {}", addr);
            Server::bind(&addr)
                .serve(router.into_make_service())
                .await
                .context("failed to start a server")?;
            Err(anyhow::anyhow!("server exited expectedly"))
        }
    });
    tokio::select! {
        web = web => {
            web??
        }
        _ = tokio::signal::ctrl_c() => {
            Err(anyhow::anyhow!("received ctrl-c event"))
        }
    }
}
