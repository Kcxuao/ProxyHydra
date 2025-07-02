use crate::model::ProxyBasic;
use anyhow::Result;
use tracing::info;

pub async fn fetch() -> Result<Vec<ProxyBasic>> {
    let text = reqwest::get("https://www.bfbke.com/proxy.txt").await?.text().await?;
    let mut list = Vec::new();

    for line in text.lines() {
        let mut parts = line.split(':');
        if let (Some(ip), Some(port)) = (parts.next(), parts.next()) {
            list.push(ProxyBasic::new(ip.to_string(), port.to_string()));
        }
    }

    info!("BFBKE 抓取了 {} 条代理", list.len());
    Ok(list)
}
