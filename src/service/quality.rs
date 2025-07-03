#![allow(dead_code)]
#![allow(unused_variables)]

//! # quality 模块
//!
//! 提供代理质量评估相关逻辑，包括测速、成功率计算、稳定性分析与综合评分。
//!
//! ## 功能简介
//!
//! - 向指定目标地址发起多轮请求，评估代理连接的成功率与速度；
//! - 计算响应时间的平均值与方差，以评估稳定性；
//! - 合并多个目标节点的测试结果，生成综合质量报告；
//! - 根据测试数据打分，生成综合评分，供筛选与排序使用。
//!
//! ## 核心结构与函数
//!
//! - [`QualityTestResults`]：单个测试任务的统计结果；
//! - [`QualityConfig`]：质量测试参数配置；
//! - [`run_tests`]：对代理执行多个目标的质量测试；
//! - [`evaluate`]：入口函数，运行测试并生成完整代理对象（含质量信息）。
//!
//! ## 使用场景
//!
//! 用于批量代理验证场景中的质量评估步骤，适合代理池清洗、优选策略、自动下线低质量节点等需求。

use crate::common::utils::{round2, speed_to_score};
use crate::db::get_storage;
use crate::db::manager::ProxyStorage;
use crate::model::{APP_CONFIG, Proxy, ProxyBasic, ProxyCheckResult};
use anyhow::Result;
use chrono::Utc;
use std::time::Duration;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tokio::time::sleep;
use tracing::log::{debug, info, warn};

/// 用于配置代理质量评估的权重与测试参数。
///
/// 包括成功率、响应速度、稳定性的权重比例，
/// 以及测试次数、单次请求超时时间和测试目标 URL 列表。
#[derive(Clone, Debug)]
pub struct QualityConfig {
    /// 速度评分权重（0.0 - 1.0）。
    pub speed_weight: f64,
    /// 成功率评分权重（0.0 - 1.0）。
    pub success_weight: f64,
    /// 稳定性评分权重（0.0 - 1.0）。
    pub stability_weight: f64,
    /// 每个代理测试的请求次数。
    pub test_count: u64,
    /// 每个代理测试的失败重试次数。
    pub max_retries: u8,
    /// 每次请求的超时时间。
    pub timeout: Duration,
    /// 用于测试的目标 URL 列表。
    pub test_urls: Vec<String>,
    /// 验证等级：快速、标准、细致
    pub verify_level: VerifyLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum VerifyLevel {
    Fast,
    Standard,
    Detailed,
}
/// 提供默认配置：
/// - 速度、成功率、稳定性权重均为 1.0；
/// - 测试次数为 3；
/// - 超时时间为 5 秒；
/// - 默认测试地址为 `https://cip.cc`。
impl Default for QualityConfig {
    fn default() -> Self {
        let level = match APP_CONFIG.verify.verify_level {
            0 => VerifyLevel::Fast,
            1 => VerifyLevel::Standard,
            2 => VerifyLevel::Detailed,
            _ => VerifyLevel::Standard,
        };

        let (test_count, max_retries, timeout) = match level {
            VerifyLevel::Fast => (1, 0, Duration::from_secs(3)),
            VerifyLevel::Standard => (3, 3, Duration::from_secs(APP_CONFIG.verify.timeout)),
            VerifyLevel::Detailed => (5, 5, Duration::from_secs(APP_CONFIG.verify.timeout * 2)),
        };

        Self {
            speed_weight: 0.4,
            success_weight: 0.3,
            stability_weight: 0.3,
            test_count,
            max_retries,
            timeout,
            test_urls: APP_CONFIG.verify.test_urls.clone(),
            verify_level: level,
        }
    }
}

/// 记录代理在多个测试中的响应时间与成功情况。
///
/// 用于计算平均速度、成功率与稳定性。
struct QualityTestResults {
    successes: Vec<f64>,
    failures: u64,
    total: u64,
}

impl QualityTestResults {
    /// 创建新的测试结果统计器。
    ///
    /// # 参数
    /// - `total`: 计划进行的总测试次数。
    fn new(total: u64) -> Self {
        Self {
            successes: Vec::new(),
            failures: 0,
            total,
        }
    }

    /// 记录一次成功的代理请求。
    ///
    /// # 参数
    /// - `duration`: 本次请求的耗时（单位：秒）。
    fn record_success(&mut self, d: f64) {
        self.successes.push(d);
    }

    /// 记录一次失败的代理请求。
    fn record_failure(&mut self) {
        self.failures += 1;
    }

    /// 计算测试中的成功率（成功次数 / 总次数）。
    ///
    /// # 返回
    /// 成功率（0.0 ~ 1.0）。
    fn success_rate(&self) -> f64 {
        let rate = if self.total == 0 {
            0.0
        } else {
            self.successes.len() as f64 / self.total as f64
        };

        round2(rate)
    }

    /// 计算所有成功请求的平均响应时间（单位：秒）。
    ///
    /// 若无成功记录则返回 `None`。
    fn average_speed(&self) -> f64 {
        let speed = if self.successes.is_empty() {
            0.0
        } else {
            self.successes.iter().map(|&x| x).sum::<f64>() / self.successes.len() as f64
        };

        round2(speed)
    }
}

/// 对单个代理进行多次测试，并根据响应情况计算评分。
///
/// 会使用 `test_count` 指定的次数对代理进行连接，
/// 统计成功率、平均速度、稳定性，并根据配置计算最终得分。
///
/// # 参数
/// - `proxy`: 待测试的代理
/// - `config`: 质量评估配置
///
/// # 返回
/// 带有打分结果的完整 `Proxy` 实例。
pub async fn evaluate(proxy: &ProxyBasic, config: &QualityConfig) -> Result<Proxy> {
    let mut result = ProxyCheckResult::default();
    let test_results = run_tests(proxy, config).await?;

    result.speed = Some(test_results.average_speed());
    result.success_rate = Some(test_results.success_rate());
    result.last_checked = Some(Utc::now().naive_utc());

    if let Some(old) = get_storage()
        .find_proxy_by_ip_port(&proxy.ip, &proxy.port)
        .await?
    {
        let delta = (result.success_rate.unwrap_or(0.0) - old.success_rate.unwrap_or(0.0)).abs();
        let stability = old.stability.unwrap_or(0.5) * 0.7 + (1.0 - delta) * 0.3;
        result.stability = Some(stability.clamp(0.0, 1.0));
    } else {
        result.stability = Some(0.5);
    }

    compute_score(&mut result, config);
    Ok(Proxy::from_parts(proxy.clone(), result))
}

/// 对给定代理执行多个目标地址的多轮请求测试，
/// 记录每轮成功率、平均速度、稳定性等指标。
///
/// 每个目标地址将进行 `test_count` 次测试，
/// 所有目标地址的结果将被合并为整体评估结果。
///
/// # 参数
/// - `proxy`: 待测试的代理基本信息（IP 和端口）
/// - `config`: 质量测试配置，包括测试次数、超时、测试地址等
///
/// # 返回
/// 返回一个合并后的 `QualityTestResults`，
/// 包含所有测试节点的综合成功率、平均响应时间与稳定性。
///
/// # 错误
/// - 若 `proxy` 构建或 HTTP 客户端构建失败，返回对应错误。
/// - 请求目标地址失败不会中断流程，只计为失败记录。
async fn run_tests(proxy: &ProxyBasic, config: &QualityConfig) -> Result<QualityTestResults> {
    let proxy_url = format!("http://{}:{}", proxy.ip, proxy.port);
    let proxy_obj = reqwest::Proxy::all(&proxy_url)?;
    let client = reqwest::Client::builder()
        .proxy(proxy_obj)
        .timeout(config.timeout)
        .build()?;

    let mut futs = FuturesUnordered::new();
    let total_tests = config.test_urls.len() as u64 * config.test_count;

    for test_url in &config.test_urls {
        for _ in 0..config.test_count {
            let client = client.clone();
            let url = test_url.clone();
            let label = format!("[{}:{}]", proxy.ip, proxy.port);

            futs.push(async move {
                send_with_retries(&client, &url, config.max_retries, &label).await
            });
        }
    }

    let mut results = QualityTestResults::new(total_tests);

    while let Some(res) = futs.next().await {
        match res {
            Some(duration) => results.record_success(duration),
            None => results.record_failure(),
        }
    }

    Ok(results)
}

/// 向指定 URL 发送 GET 请求，失败时进行最多 `max_retries` 次重试，并记录耗时。
///
/// 每次请求都会打印日志，包括成功、失败和状态码错误的信息，方便调试和跟踪代理质量。
///
/// # 参数
/// - `client`: 配置好的 `reqwest::Client`，包含代理设置与超时。
/// - `url`: 要请求的目标 URL 字符串。
/// - `max_retries`: 最大重试次数（不包括第一次尝试）。
/// - `label`: 用于日志输出的代理标签（例如 `[127.0.0.1:8080]`）。
///
/// # 返回
/// - `Some(f64)`：请求成功时返回耗时（单位：秒，保留两位小数）。
/// - `None`：请求全部失败或状态码非 2xx。
///
/// # 日志输出示例
/// ```text
/// 🔁 [127.0.0.1:8080] 第 1 次请求 https://example.com 失败，原因：连接超时，正在重试...
/// ❌ [127.0.0.1:8080] 第 3 次请求 https://example.com 最终失败，原因：连接被拒绝
/// ```
async fn send_with_retries(
    client: &reqwest::Client,
    url: &str,
    max_retries: u8,
    label: &str, // 用于输出代理 IP 信息
) -> Option<f64> {
    let mut attempt = 0;
    let mut backoff = Duration::from_millis(500);

    while attempt <= max_retries {
        debug!(
            "{} 开始第 {} 次请求 {}",
            label,
            attempt + 1,
            url
        );

        let start = std::time::Instant::now();
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let elapsed = start.elapsed().as_secs_f64();
                debug!(
                    "{} 第 {} 次请求 {} 成功，耗时 {:.2} 秒",
                    label,
                    attempt + 1,
                    url,
                    elapsed
                );
                return Some(round2(elapsed));
            }
            Err(e) => {
                debug!(
                    "🔁 {} 第 {} 次请求 {} 失败，原因：{}",
                    label,
                    attempt + 1,
                    url,
                    e
                );
                if attempt < max_retries {
                    debug!("{} 正在等待 {:?} 后重试...", label, backoff);
                    sleep(backoff).await;
                    backoff *= 2; // 指数退避
                } else {
                    debug!(
                        "❌ {} 第 {} 次请求 {} 最终失败，原因：{}",
                        label,
                        attempt + 1,
                        url,
                        e
                    );
                }
            }
            Ok(resp) => {
                debug!(
                    "⚠️ {} 第 {} 次请求 {} 返回非成功状态：{}",
                    label,
                    attempt + 1,
                    url,
                    resp.status()
                );
                if attempt < max_retries {
                    debug!("{} 返回状态异常，等待 {:?} 后重试...", label, backoff);
                    sleep(backoff).await;
                    backoff *= 2;
                } else {
                    debug!(
                        "❌ {} 第 {} 次请求 {} 最终返回非成功状态：{}",
                        label,
                        attempt + 1,
                        url,
                        resp.status()
                    );
                }
            }
        }
        attempt += 1;
    }

    None
}

/// 合并多个测试结果为一个整体测试统计。
///
/// 会将所有成功响应时间合并，
/// 同时累计失败次数与总测试次数，
/// 用于统一评估多个测试地址的质量表现。
///
/// # 参数
/// - `results`: 各目标地址的单独测试结果集合
///
/// # 返回
/// 一个合并后的 `QualityTestResults` 实例。
fn merge_test_results(results: &[QualityTestResults]) -> QualityTestResults {
    let total_tests = results.iter().map(|r| r.total).sum();
    let mut merged = QualityTestResults::new(total_tests);

    for r in results {
        merged.successes.extend(&r.successes);
        merged.failures += r.failures;
    }

    merged
}

/// 根据配置权重计算代理最终综合评分。
///
/// 若某一维度值缺失，将其视为 0 参与加权。
///
/// # 参数
/// - `proxy`: 已评估的代理结果（字段将被修改）
/// - `config`: 权重配置
fn compute_score(proxy: &mut ProxyCheckResult, config: &QualityConfig) {
    let speed_score = speed_to_score(proxy.speed.unwrap_or(f64::MAX));
    let success = proxy.success_rate.unwrap_or(0.0);
    let stability = proxy.stability.unwrap_or(0.0);

    proxy.score = Some(
        (speed_score * config.speed_weight
            + success * config.success_weight
            + stability * config.stability_weight)
            .clamp(0.0, 1.0),
    );
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::model::ProxyBasic;
    use crate::service::quality::QualityConfig;

    #[tokio::test]
    async fn test_evaluate() {
        db::init().await.unwrap();
        let basic = ProxyBasic::new("127.0.0.1", "12334");
        let config = QualityConfig::default();

        let proxy = super::evaluate(&basic, &config).await.unwrap();
        assert_eq!(proxy.ip, basic.ip);
    }
}
