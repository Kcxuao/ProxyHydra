//! # verifier 模块
//!
//! 用于验证代理的可用性与质量。
//!
//! 本模块提供以下功能：
//!
//! - 批量去重并验证代理的连通性和质量表现；  
//! - 使用 `quality` 模块对每个代理进行测速与成功率评估；  
//! - 将有效代理（成功率 > 0）写入数据库；  
//! - 支持并发控制（通过信号量限制并发请求数量）；  
//! - 输出验证过程的详细日志与统计信息。
//!
//! ## 主要函数
//!
//! - verify_all：验证整个代理列表，返回成功数量；  
//! - verify_single：验证单个代理是否可用（私有辅助函数）。
//!
//! ## 使用场景
//!
//! 可用于定期清洗代理池、筛选高质量代理、构建代理服务数据源。


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
use crate::db::get_storage;
use crate::db::manager::ProxyStorage;

/// 批量验证多个代理的可用性，并统计验证成功的代理数量。
///
/// 此函数将：
/// 1. 对传入代理列表去重；
/// 2. 并发限制地执行每个代理的质量评估（包括测速与稳定性测试）；
/// 3. 若评估通过（成功率 > 0），则写入存储；
/// 4. 最终返回成功验证的代理数量。
///
/// 日志将记录验证过程及结果统计。
///
/// # 参数
/// - `basics`: 原始代理列表（含 IP 与端口）
///
/// # 返回
/// 成功验证并写入的代理数量。
///
/// # 错误
/// 如果在代理验证过程中发生错误（如网络异常、数据库写入失败），将返回对应错误。
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


/// 验证单个代理的有效性。
///
/// 该函数将对代理进行质量评估（包括测速、成功率与稳定性），
/// 并根据成功率判断其是否为有效代理：
/// - 若成功率大于 0，将其写入数据库并返回 `true`；
/// - 否则返回 `false`。
///
/// # 参数
/// - `basic`: 代理基本信息（IP 和端口）
/// - `config`: 质量评估配置参数
///
/// # 返回
/// `Ok(true)` 表示验证通过；`Ok(false)` 表示验证失败。
/// 若发生错误（如请求失败、存储异常），则返回 `Err(ApiError)`。
async fn verify_single(basic: &ProxyBasic, config: &quality::QualityConfig) -> Result<bool> {
    // 调用质量评估，返回完整 Proxy（带质量信息）
    let updated: Proxy = quality::evaluate(basic, config).await?;

    // 只要成功率大于0就认为有效，存储数据库
    if updated.success_rate.unwrap_or(0.0) > 0.0 {
        get_storage().upsert_quality_proxy(&updated).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::model::ProxyBasic;
    use crate::service::quality::QualityConfig;

    #[tokio::test]
    async fn test_verify_all() {
        db::init().await.unwrap();
        
        let list = vec![
            ProxyBasic::new("127.0.0.1", "12334"), 
            ProxyBasic::new("127.0.0.1", "12335"),
        ];
        
        let result = super::verify_all(list).await.unwrap();
        assert!(result > 0);
    }

    #[tokio::test]
    async fn test_verify_single() {
        db::init().await.unwrap();

        let proxy = ProxyBasic::new("127.0.0.1", "12334");
        let config = QualityConfig::default();
        
        let result = super::verify_single(&proxy, &config).await.unwrap();
        assert!(result);
    }
}
