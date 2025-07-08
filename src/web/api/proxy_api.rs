use rand::Rng;
use crate::model::{Proxy, ProxyBasic};
use salvo::prelude::*;
use crate::common::cache::CACHE;
use crate::service::verifier::{verify_all, verify_database};
use crate::web::hoops::cache::cache_proxies;

#[handler]
async fn get_proxy() -> Json<ProxyBasic> {
    let proxies = CACHE.get(&"proxies").unwrap();
    let r = rand::rng().random_range(0..proxies.len());
    Json(proxies[r].basic())
}

#[handler]
async fn verify_proxy() -> String {
    verify_database().await.unwrap();
    "数据库存活代理开始校验".to_string()
}

#[handler]
async fn list_proxy() -> Json<Vec<ProxyBasic>> {
    let proxies = CACHE.get(&"proxies").unwrap();
    let list: Vec<ProxyBasic> = proxies.iter().map(|p| p.basic()).collect();
    Json(list)
}

pub fn proxy_router() -> Router {
    Router::with_path("proxy")
        .hoop(cache_proxies)
        .get(get_proxy)
        .push(
            Router::with_path("list")
            .get(list_proxy)
        )
        .push(
            Router::with_path("verify")
                .get(verify_proxy)
        )
}
