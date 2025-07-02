use crate::{error::ApiError, model::{Proxy, APP_CONFIG}, storage};
use anyhow::Result;
use chrono::Local;
use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct QualityConfig {
    pub speed_weight: f32,
    pub success_weight: f32,
    pub stability_weight: f32,
    pub test_count: u32,
    pub timeout: Duration,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            speed_weight: 0.4,
            success_weight: 0.3,
            stability_weight: 0.3,
            test_count: 3,
            timeout: Duration::from_secs(APP_CONFIG.timeout),
        }
    }
}

pub async fn evaluate(proxy: &Proxy, config: &QualityConfig) -> Result<Proxy, ApiError> {
    let mut result = proxy.clone();
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
    Ok(result)
}

async fn run_tests(proxy: &Proxy, config: &QualityConfig) -> Result<QualityTestResults> {
    let proxy_url = format!("http://{}:{}", proxy.ip, proxy.port);
    let proxy_obj = reqwest::Proxy::all(&proxy_url)?;

    let mut results = QualityTestResults::new(config.test_count);

    for _ in 0..config.test_count {
        let client = Client::builder()
            .proxy(proxy_obj.clone())
            .timeout(config.timeout)
            .build()?;

        let start = std::time::Instant::now();
        match client.get("https://cip.cc").send().await {
            Ok(_) => {
                let elapsed = start.elapsed().as_secs_f32(); // 获取秒
                let rounded = (elapsed * 100.0).round() / 100.0; // 保留两位小数
                results.record_success(rounded);
            }
            Err(_) => {
                results.record_failure();
            }
        }
    }
    Ok(results)
}

fn compute_score(proxy: &mut Proxy, config: &QualityConfig) {
    let speed_score = match proxy.speed.unwrap_or(f32::MAX) {
        s if s < 100.0 => 1.0,
        s if s < 500.0 => 0.8,
        s if s < 1000.0 => 0.5,
        s if s < 2000.0 => 0.3,
        _ => 0.1,
    };
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

        Self::round2(rate)
    }

    fn average_speed(&self) -> f32 {
        let speed = if self.successes.is_empty() {
            0.0
        } else {
            self.successes.iter().map(|&x| x as f32).sum::<f32>() / self.successes.len() as f32
        };

        Self::round2(speed)
    }

    fn round2(val: f32) -> f32 {
        (val * 100.0).round() / 100.0
    }
}
