use crate::model::ProxyBasic;
use anyhow::Result;
use tracing::info;

/// 从 BFBKE 网站抓取代理列表，并解析为 `ProxyBasic` 向量。
///
/// 该函数会向 `https://www.bfbke.com/proxy.txt` 发起 GET 请求，
/// 解析返回的文本数据，将每一行按 `ip:port` 格式拆分，并构建对应的代理条目。
///
/// # 返回
/// 返回一个包含所有有效解析结果的 `Vec<ProxyBasic>`。
///
/// # 错误
/// 若网络请求失败，或响应格式不符合预期（如某行缺失冒号），将返回相应的错误。
///
/// # 示例返回格式（原始数据）
/// ```text
/// 123.45.67.89:8080
/// 98.76.54.32:3128
/// ```
///
/// # 日志
/// 会输出抓取到的代理数量日志：`BFBKE 抓取了 N 条代理`。
pub async fn fetch() -> Result<Vec<ProxyBasic>> {
    let text = reqwest::get("https://www.bfbke.com/proxy.txt").await?.text().await?;
    let mut list = Vec::new();

    for line in text.lines() {
        let mut parts = line.split(':');
        if let (Some(ip), Some(port)) = (parts.next(), parts.next()) {
            list.push(ProxyBasic::new(ip, port));
        }
    }

    info!("BFBKE 抓取了 {} 条代理", list.len());
    Ok(list)
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_fetch() {
        let list = super::fetch().await.unwrap();
        assert!(list.len() > 0);
    }
}
