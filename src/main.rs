mod model;
mod service;
mod fetcher;
mod common;
mod db;

use tracing::log::info;
use tracing::{error, warn};
use tracing_subscriber::fmt::init;
use crate::common::log::init_logging;
use crate::service::verifier;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 必须是程序第一个调用！
    init_logging().expect("Failed to initialize logging");
    // db::init().await?; // 初始化数据库
    tracing::error!("这是 error 级别");
    tracing::warn!("这是 warn 级别");
    tracing::info!("这是 info 级别");
    tracing::debug!("这是 debug 级别");
    tracing::trace!("这是 trace 级别");

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

    Ok(())
}
