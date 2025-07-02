use crate::{error::ApiError, storage};
use anyhow::{anyhow, Result};
use chrono::Local;
use reqwest::Client;
use std::time::Duration;
use crate::model::{Proxy, ProxyBasic, ProxyCheckResult, APP_CONFIG};
use crate::utils::{round2, speed_to_score};

#[derive(Clone)]
pub struct QualityConfig {
    pub speed_weight: f32,
    pub success_weight: f32,
    pub stability_weight: f32,
    pub test_count: u32,
    pub timeout: Duration,
    pub test_urls: Vec<String>,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            speed_weight: 0.4,
            success_weight: 0.3,
            stability_weight: 0.3,
            test_count: 3,
            timeout: Duration::from_secs(APP_CONFIG.verify.timeout),
            test_urls: APP_CONFIG.verify.test_urls.clone(),
        }
    }
}

pub async fn evaluate(proxy: &ProxyBasic, config: &QualityConfig) -> Result<Proxy, ApiError> {
    let mut result = ProxyCheckResult::default();
    let test_results = run_tests(proxy, config).await?;

    result.speed = Some(test_results.average_speed());
    result.success_rate = Some(test_results.success_rate());
    result.last_checked = Some(Local::now().to_rfc3339());

    if let Some(old) = storage::find_proxy_by_ip_port(&proxy.ip, &proxy.port).await? {
        let delta = (result.success_rate.unwrap() - old.success_rate.unwrap_or(0.0)).abs();
        let stability = old.stability.unwrap_or(0.5) * 0.7 + (1.0 - delta) * 0.3;
        result.stability = Some(stability.clamp(0.0, 1.0));
    } else {
        result.stability = Some(0.5);
    }

    compute_score(&mut result, config);
    Ok(Proxy::from_parts(proxy.clone(), result))
}

async fn run_tests(proxy: &ProxyBasic, config: &QualityConfig) -> Result<QualityTestResults> {
    let proxy_url = format!("http://{}:{}", proxy.ip, proxy.port);
    let proxy_obj = reqwest::Proxy::all(&proxy_url)?;

    let mut all_results = Vec::new();

    for test_url in &config.test_urls {
        let mut results = QualityTestResults::new(config.test_count);

        for _ in 0..config.test_count {
            let client = reqwest::Client::builder()
                .proxy(proxy_obj.clone())
                .timeout(config.timeout)
                .build()?;

            let start = std::time::Instant::now();
            match client.get(test_url).send().await {
                Ok(_) => {
                    let elapsed = start.elapsed().as_secs_f32();
                    let rounded = (elapsed * 100.0).round() / 100.0;
                    results.record_success(rounded);
                }
                Err(_) => {
                    results.record_failure();
                }
            }
        }
        all_results.push(results);
    }

    // 综合所有节点结果
    Ok(merge_test_results(&all_results))
}

fn merge_test_results(results: &[QualityTestResults]) -> QualityTestResults {
    let total_tests = results.iter().map(|r| r.total).sum();
    let mut merged = QualityTestResults::new(total_tests);

    for r in results {
        merged.successes.extend(&r.successes);
        merged.failures += r.failures;
    }

    merged
}

/// 速度打分等级示意：
/// <100ms = 1.0
/// <500ms = 0.8
/// <1000ms = 0.5
/// <2000ms = 0.3
/// ≥2000ms = 0.1
fn compute_score(proxy: &mut ProxyCheckResult, config: &QualityConfig) {
    let speed_score = speed_to_score(proxy.speed.unwrap_or(f32::MAX));
    let success = proxy.success_rate.unwrap_or(0.0);
    let stability = proxy.stability.unwrap_or(0.0);

    proxy.score = Some(
        speed_score * config.speed_weight
            + success * config.success_weight
            + stability * config.stability_weight,
    );
}

struct QualityTestResults {
    successes: Vec<f32>,
    failures: u32,
    total: u32,
}

impl QualityTestResults {
    fn new(total: u32) -> Self {
        Self {
            successes: Vec::new(),
            failures: 0,
            total,
        }
    }

    fn record_success(&mut self, d: f32) {
        self.successes.push(d);
    }
    fn record_failure(&mut self) {
        self.failures += 1;
    }

    fn success_rate(&self) -> f32 {
        let rate = if self.total == 0 {
            0.0
        } else {
            self.successes.len() as f32 / self.total as f32
        };

        round2(rate)
    }

    fn average_speed(&self) -> f32 {
        let speed = if self.successes.is_empty() {
            0.0
        } else {
            self.successes.iter().map(|&x| x).sum::<f32>() / self.successes.len() as f32
        };

        round2(speed)
    }
}

impl Default for ProxyCheckResult {
    fn default() -> Self {
        Self {
            speed: None,
            success_rate: None,
            stability: None,
            score: None,
            last_checked: None,
        }
    }
}
