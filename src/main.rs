mod model;
mod service;
mod fetcher;
mod common;
mod db;
mod web;

use crate::common::log::init_logging;
use crate::db::manager::ProxyStorage;
use crate::web::api::proxy_api::proxy_router;
use salvo::prelude::TcpListener;
use salvo::{Listener, Router, Server};
use crate::model::APP_CONFIG;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging().expect("Failed to initialize logging");
    db::init().await?; // 初始化数据库

    let acceptor = TcpListener::new(format!("{}:{}", APP_CONFIG.server.addr, APP_CONFIG.server.port)).bind().await;

    let router = Router::new().push(proxy_router());
    Server::new(acceptor).serve(router).await;

    Ok(())
}
