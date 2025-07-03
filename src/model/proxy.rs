#![allow(dead_code)]
#![allow(unused_variables)]

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Proxy {
    /// 代理的 IP 地址（IPv4 或 IPv6）。
    pub ip: String,

    /// 代理的端口号（字符串形式，便于处理）。
    pub port: String,

    /// 平均响应速度（单位：秒），从多个测试请求中得出。
    ///
    /// 若未进行测速，该字段为 `None`。
    pub speed: Option<f64>,

    /// 成功率，表示请求成功次数占总请求次数的比例（范围 0.0 - 1.0）。
    ///
    /// 若未进行测试，该字段为 `None`。
    pub success_rate: Option<f64>,

    /// 稳定性分数，反映响应时间的一致性（如标准差或方差反比）。
    ///
    /// 分值越高表示响应越稳定。若未测试，该字段为 `None`。
    pub stability: Option<f64>,

    /// 综合评分，基于成功率、速度和稳定性计算得出。
    ///
    /// 用于排序和筛选高质量代理。若尚未评分，则为 `None`。
    pub score: Option<f64>,

    /// 最近一次进行质量检测的时间。
    ///
    /// 若代理尚未被验证或存储前未检测，则为 `None`。
    pub last_checked: Option<NaiveDateTime>,
}

impl Proxy {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            ip,
            port,
            speed: None,
            success_rate: None,
            stability: None,
            score: None,
            last_checked: None,
        }
    }

    pub fn basic(&self) -> ProxyBasic {
        ProxyBasic {
            ip: self.ip.clone(),
            port: self.port.clone(),
        }
    }

    pub fn result(&self) -> ProxyCheckResult {
        ProxyCheckResult {
            speed: self.speed,
            success_rate: self.success_rate,
            stability: self.stability,
            score: self.score,
            last_checked: self.last_checked,
        }
    }

    pub fn from_parts(basic: ProxyBasic, result: ProxyCheckResult) -> Self {
        Self {
            ip: basic.ip,
            port: basic.port,
            speed: result.speed,
            success_rate: result.success_rate,
            stability: result.stability,
            score: result.score,
            last_checked: result.last_checked,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProxyBasic {
    pub ip: String,
    pub port: String,
}

impl ProxyBasic {
    pub fn new(ip: &str, port: &str) -> Self {
        ProxyBasic { ip: ip.to_string(), port: port.to_string() }
    }

    pub fn is_none_empty(&self) -> bool {
        self.ip.is_empty() && self.port.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProxyCheckResult {
    pub speed: Option<f64>,
    pub success_rate: Option<f64>,
    pub stability: Option<f64>,
    pub score: Option<f64>,
    pub last_checked: Option<NaiveDateTime>,
}
