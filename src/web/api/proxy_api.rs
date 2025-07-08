use anyhow::{anyhow, Context};
use crate::common::cache::CACHE;
use crate::model::ProxyBasic;
use crate::service::verifier::verify_database;
use crate::web::hoops::cache::cache_proxies;
use rand::Rng;
use salvo::prelude::*;
use tracing::log::info;
use crate::fetcher;
use crate::service::verifier;

#[handler]
async fn get_proxy() -> anyhow::Result<Json<ProxyBasic>> {
    let proxies = CACHE.get(&"proxies").context("代理缓存获取失败")?;
    let r = rand::rng().random_range(0..proxies.len());
    Ok(Json(proxies[r].basic()))
}

#[handler]
async fn verify_proxy() -> String {
    verify_database().await.unwrap();
    "数据库存活代理校验完成".to_string()
}

#[handler]
async fn proxy_collection() -> anyhow::Result<()> {
    info!("========== [代理采集阶段] ==========");
    let list = fetcher::fetch_all_sources().await?;
    tracing::info!("抓取到总共 {} 条代理", list.len());
    verifier::verify_all(list).await?;

    Ok(())
}

#[handler]
async fn list_proxy() -> anyhow::Result<Json<Vec<ProxyBasic>>> {
    let proxies = CACHE.get(&"proxies").context("代理缓存获取失败")?;
    let list: Vec<ProxyBasic> = proxies.iter().map(|p| p.basic()).collect();
    Ok(Json(list))
}

pub fn proxy_router() -> Router {
    Router::with_path("proxy")
        .hoop(cache_proxies)
        .get(get_proxy)
        .push(Router::with_path("list").get(list_proxy))
        .push(Router::with_path("verify").get(verify_proxy))
        .push(Router::with_path("collection").get(proxy_collection))
}
