use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Deserialize, Serialize)]
pub struct Proxy {
    /// [主键] 代理记录ID（SQLite类型：INTEGER）
    /// - 自动递增，插入时设为None即可
    /// - 示例：`Some(1)`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<f32>,

    /// 代理服务器IP地址（SQLite类型：TEXT NOT NULL）
    /// - 格式：IPv4/IPv6地址
    /// - 示例：`"192.168.1.100"`
    pub ip: String,

    /// 代理服务器端口（SQLite类型：TEXT NOT NULL）
    /// - 范围：1-65535
    /// - 示例：`"8080"`
    pub port: String,

    /// 平均响应速度（SQLite类型：REAL）
    /// - 单位：毫秒(ms)
    /// - 通过最近3次测试计算平均值
    /// - 示例：`Some(256.8)`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,

    /// 请求成功率（SQLite类型：REAL）
    /// - 范围：0.0-1.0
    /// - 计算公式：成功次数/总测试次数
    /// - 示例：`Some(0.85)` 表示85%成功率
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_rate: Option<f32>,

    /// 稳定性评分（SQLite类型：REAL）
    /// - 范围：0.0-1.0
    /// - 基于历史成功率的波动计算
    /// - 示例：`Some(0.92)` 表示稳定性很好
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f32>,

    /// 综合质量评分（SQLite类型：REAL）
    /// - 范围：0.0-1.0
    /// - 计算公式：0.4*speed_score + 0.3*success_rate + 0.3*stability
    /// - 示例：`Some(0.88)`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,

    /// 最后检查时间（SQLite类型：TEXT）
    /// - 格式：ISO 8601字符串（如`"2023-08-20T14:30:00Z"`）
    /// - 自动更新为当前时间
    /// - 示例：`Some("2023-08-20T14:30:00Z".to_string())`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checked: Option<String>,
}

impl Proxy {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            id: None,
            ip,
            port,
            speed: Some(0.0),
            success_rate: Some(0.0),
            stability: Some(0.0),
            score: Some(0.0),
            last_checked: Some("".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub semaphore: usize,
    pub timeout: u64,
}

pub static APP_CONFIG: Lazy<AppConfig> =
    Lazy::new(|| AppConfig::load().expect("Failed to load configuration"));

impl AppConfig {
    fn load() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("Config"))
            .build()
            .unwrap();
        let config = config.try_deserialize()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        println!("{:#?}", APP_CONFIG.semaphore);
    }
}
