use crate::storage;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc
};
use anyhow::Result;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::{error, info};
use tracing::log::warn;
use crate::common::error::ApiError;
use crate::model::{Proxy, ProxyBasic, APP_CONFIG};
use crate::service::quality;
use crate::common::utils::dedup_proxies;


pub async fn verify_all(mut basics: Vec<ProxyBasic>) -> Result<usize> {
    info!("========== [代理去重阶段] ==========");
    basics = dedup_proxies(basics);
    let len = basics.len();

    info!("========== [代理验证阶段] ==========");
    info!("🚀 开始批量验证代理，共 {} 条待验证", len);

    let success_count = Arc::new(AtomicUsize::new(0));
    let semaphore = Arc::new(Semaphore::new(APP_CONFIG.verify.semaphore));
    let quality_config = quality::QualityConfig::default();

    let tasks: Vec<_> = basics.into_iter().enumerate().map(|(i, basic)| {
        let success_count = Arc::clone(&success_count);
        let semaphore = Arc::clone(&semaphore);
        let quality_config = quality_config.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.unwrap();
            let start = Instant::now();

            let label = format!("[#{} {}:{}]", i + 1, basic.ip, basic.port);

            // 🛰️ 打印参与测速的目标节点地址
            let nodes = quality_config.test_urls.join(", ");
            info!("📡 {} 开始验证，测速节点：{}", label, nodes);

            match verify_single(&basic, &quality_config).await {
                Ok(true) => {
                    success_count.fetch_add(1, Ordering::SeqCst);
                    let ms = start.elapsed().as_millis();
                    info!("🟢 {} 验证通过，耗时 {}ms", label, ms);
                }
                Ok(false) => {
                    let ms = start.elapsed().as_millis();
                    warn!("🔴 {} 无效代理，耗时 {}ms", label, ms);
                }
                Err(e) => {
                    let ms = start.elapsed().as_millis();
                    error!("❌ {} 验证出错，耗时 {}ms，错误：{}", label, ms, e);
                }
            }

            Ok::<(), ApiError>(())
        })
    }).collect();

    for task in tasks {
        task.await??;
    }

    let ok = success_count.load(Ordering::SeqCst);
    info!("========== [结果统计完成 ✅] ==========");
    info!("✅ 验证完成：总计 {} 条，成功 {} 条，失败 {} 条", len, ok, len - ok);

    Ok(ok)
}


/// 验证单个代理
async fn verify_single(basic: &ProxyBasic, config: &quality::QualityConfig) -> Result<bool, ApiError> {
    // 调用质量评估，返回完整 Proxy（带质量信息）
    let updated: Proxy = quality::evaluate(basic, config).await?;

    // 只要成功率大于0就认为有效，存储数据库
    if updated.success_rate.unwrap_or(0.0) > 0.0 {
        storage::upsert_quality_proxy(&updated).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
