use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Proxy {
    pub ip: String,
    pub port: String,
    pub speed: Option<f32>,
    pub success_rate: Option<f32>,
    pub stability: Option<f32>,
    pub score: Option<f32>,
    pub last_checked: Option<String>,
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
            last_checked: self.last_checked.clone(),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyBasic {
    pub ip: String,
    pub port: String,
}

impl ProxyBasic {
    pub fn new(ip: String, port: String) -> Self {
        ProxyBasic { ip, port }
    }

    pub fn is_none_empty(&self) -> bool {
        self.ip.is_empty() && self.port.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ProxyCheckResult {
    pub speed: Option<f32>,
    pub success_rate: Option<f32>,
    pub stability: Option<f32>,
    pub score: Option<f32>,
    pub last_checked: Option<String>,
}
