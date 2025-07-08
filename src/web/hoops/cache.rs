use anyhow::anyhow;
use salvo::{handler, Depot};
use tracing::info;
use crate::common::cache::{GlobalCache, CACHE};
use crate::db::get_storage;
use crate::db::manager::ProxyStorage;
use crate::model::Proxy;

#[handler]
pub async fn cache_proxies() {
    match CACHE.get(&"proxies") {
        Some(proxies) => proxies,
        None => {
            // 从存储加载并更新缓存
            info!("Proxies Cache...");
            let proxies = get_storage().list_all_proxies().await.unwrap();
            CACHE.set("proxies", proxies.clone());
            proxies
        }
    };
}
