#![allow(dead_code)]
#![allow(unused_variables)]

use crate::model::Proxy;
use anyhow::Result;
use regex::Regex;
use tracing::info;

mod sources {
    pub const KUAI_DAILI: &'static str = "https://www.kuaidaili.com/free/intr/{}/";
    pub const BFBKE: &'static str = "https://www.bfbke.com/proxy.txt";
}

pub async fn fetch_all_sources() -> Result<Vec<Proxy>> {
    let mut proxies = Vec::new();

    proxies.extend(fetch_from_bfbke().await?);
    // proxies.extend(fetch_from_kuai().await?);

    Ok(proxies)
}

async fn fetch_from_bfbke() -> Result<Vec<Proxy>> {
    info!("Fetching proxies from BFBKE...");
    let text = reqwest::get(sources::BFBKE).await?.text().await?;
    let mut list = Vec::new();

    for line in text.lines() {
        let mut parts = line.split(':');
        if let (Some(ip), Some(port)) = (parts.next(), parts.next()) {
            if !ip.is_empty() && !port.is_empty() {
                list.push(Proxy::new(ip.to_string(), port.to_string()));
            }
        }
    }
    info!("BFBKE - got {} proxies", list.len());
    Ok(list)
}

async fn fetch_from_kuai() -> Result<Vec<Proxy>> {
    let mut list = Vec::new();
    let regex = Regex::new(r#"const fpsList = (.*);"#)?;
    for page in 1..=5 {
        info!("正在抓取快代理第 {} 页的数据...", page);
        let url = sources::KUAI_DAILI.replace("{}", &page.to_string());
        let html = reqwest::get(&url).await?.text().await?;
        if let Some(cap) = regex.captures(&html) {
            let json = cap.get(1).unwrap().as_str();
            let proxies: Vec<Proxy> = serde_json::from_str(json)?;
            info!("KuaiDaiLi page {} got {} proxies", page, proxies.len());
            list.extend(proxies);
        } else {
            info!("未解析到 fpsList 字段，可能网页结构发生变化");
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    Ok(list)
}
