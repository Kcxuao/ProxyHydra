use crate::model::ProxyBasic;
use std::collections::HashSet;
use tracing::Level;

/// 将浮点数四舍五入为两位小数。
pub fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

pub fn dedup_proxies(proxies: Vec<ProxyBasic>) -> Vec<ProxyBasic> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for proxy in proxies.into_iter() {
        let key = format!("{}:{}", proxy.ip, proxy.port);
        if seen.insert(key) {
            result.push(proxy);
        }
    }
    result
}

/// 根据速度（毫秒）计算评分，0.0~1.0。越快得分越高。
pub fn speed_to_score(speed: f64) -> f64 {
    if speed <= 0.0 {
        return 0.0;
    }

    if speed < 300.0 {
        1.0
    } else if speed < 1000.0 {
        // 300ms 到 1000ms：对数型递减
        let ratio = (speed - 300.0) / 700.0; // 归一化到 0~1
        1.0 - ratio.powf(0.5) // 平滑递减
    } else if speed < 5000.0 {
        // 1s~5s 之间：线性快速下降
        let ratio = (speed - 1000.0) / 4000.0;
        (0.3 - ratio).max(0.0) // 最多给 0.3 分
    } else {
        0.0
    }
}


/// 表名基本校验
pub fn validate_table_name(name: &str) -> bool {
    // 限定表名为英文字母、下划线、数字，且不能以数字开头
    let is_valid = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_') && name.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false);
    is_valid
}

// 把字符串转换成 Level，忽略大小写，不识别时返回 None
pub fn parse_level(s: &str) -> Option<Level> {
    match s.to_uppercase().as_str() {
        "ERROR" => Some(Level::ERROR),
        "WARN" | "WARNING" => Some(Level::WARN),
        "INFO" => Some(Level::INFO),
        "DEBUG" => Some(Level::DEBUG),
        "TRACE" => Some(Level::TRACE),
        _ => None,
    }
}

