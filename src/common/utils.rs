use std::collections::HashSet;
use crate::model::ProxyBasic;

/// 将浮点数四舍五入为两位小数。
pub fn round2(val: f32) -> f32 {
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

/// 根据响应时间（毫秒）计算速度得分（0.0 ~ 1.0）
/// 可根据项目需求自行调整评分标准
pub fn speed_to_score(speed_ms: f32) -> f32 {
    match speed_ms {
        s if s < 100.0 => 1.0,
        s if s < 500.0 => 0.8,
        s if s < 1000.0 => 0.5,
        s if s < 2000.0 => 0.3,
        _ => 0.1,
    }
}
