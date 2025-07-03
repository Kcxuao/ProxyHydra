use crate::model::ProxyBasic;
use anyhow::Result;
use regex::Regex;
use tracing::info;

pub async fn fetch() -> Result<Vec<ProxyBasic>> {
    let re = Regex::new(r#"const fpsList = (.*);"#)?;
    let mut list = Vec::new();

    for page in 1..=3 {
        info!("正在请求第 {} 页数据", page);
        let url = format!("https://www.kuaidaili.com/free/intr/{}/", page);
        let html = reqwest::get(&url).await?.text().await?;
        if let Some(cap) = re.captures(&html) {
            let json = cap.get(1).unwrap().as_str();
            let proxies: Vec<ProxyBasic> = serde_json::from_str(json)?;
            list.extend(proxies);
        }
    }

    Ok(list)
}
