use crate::{error::ApiError, model::{Proxy, APP_CONFIG}, quality, storage};
use tracing::{error, info};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use tokio::sync::Semaphore;

pub async fn verify_all(list: Vec<Proxy>) -> Result<usize, ApiError> {
    info!("开始批量验证代理可用性，共 {} 条代理", list.len());
    let count = Arc::new(AtomicUsize::new(0));
    let len = list.len();

    let sem = Arc::new(Semaphore::new(APP_CONFIG.semaphore));
    let config = quality::QualityConfig::default();

    let tasks: Vec<_> = list.into_iter().map(|p| {
        let count = count.clone();
        let sem = sem.clone();
        let config = config.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire_owned().await.unwrap();
            match verify_single(&p, &config).await {
                Ok(true) => { count.fetch_add(1, Ordering::SeqCst); }
                Ok(false) => error!("代理无效：{}:{}", p.ip, p.port),
                Err(e) => info!("代理验证失败：{}:{} - {}", p.ip, p.port, e),
            }
        })
    }).collect();

    futures::future::join_all(tasks).await;
    let ok = count.load(Ordering::SeqCst);
    info!("验证完成：共 {} 条，成功 {} 条", len, ok);
    Ok(ok)
}

async fn verify_single(proxy: &Proxy, config: &quality::QualityConfig) -> Result<bool, ApiError> {
    let updated = quality::evaluate(proxy, config).await?;
    if updated.success_rate.unwrap_or(0.0) > 0.0 {
        storage::upsert_quality_proxy(&updated).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
