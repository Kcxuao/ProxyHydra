mod model;
mod error;
mod storage;
mod fetcher;
mod quality;
mod verifier;

use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init(); // 初始化日志
    storage::init().await?; // 初始化数据库

    let list = match fetcher::fetch_all_sources().await {
        Ok(xs) => xs,
        Err(e) => {
            tracing::error!("Failed to fetch proxies: {}", e);
            return Ok(());
        }
    };
    tracing::info!("抓取到总共 {} 条代理", list.len());
    verifier::verify_all(list).await?;

    Ok(())
}
