mod model;
mod service;
mod fetcher;
mod common;
mod db;
mod web;

use std::sync::{Arc, Mutex};
use salvo::{Depot, Listener, Router, Server};
use salvo::prelude::TcpListener;
use crate::common::log::init_logging;
use crate::db::get_storage;
use crate::db::manager::ProxyStorage;
use crate::web::api::proxy_api::proxy_router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 必须是程序第一个调用！
    init_logging().expect("Failed to initialize logging");
    db::init().await?; // 初始化数据库

    // info!("========== [代理采集阶段] ==========");
    // let list = match fetcher::fetch_all_sources().await {
    //     Ok(xs) => xs,
    //     Err(e) => {
    //         tracing::error!("Failed to fetch proxies: {}", e);
    //         return Ok(());
    //     }
    // };
    // tracing::info!("抓取到总共 {} 条代理", list.len());
    // verifier::verify_all(list).await?;

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;

    let router = Router::new().push(proxy_router());
    Server::new(acceptor).serve(router).await;

    Ok(())
}
