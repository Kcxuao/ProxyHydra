use anyhow::anyhow;
use serde_json::Value;
use tracing::info;
use crate::model::ProxyBasic;

pub async fn fetch() -> anyhow::Result<Vec<ProxyBasic>> {
    info!("========== [LumiProxy] ==========");
    let mut proxies = Vec::new();
    for page in 1..=5 {
        info!("正在请求第 {} 页数据", page);
        let url = "https://api.lumiproxy.com/web_v1/free-proxy/list?page_size=60&page=1&protocol=1&anonymity=1&language=zh-hans";
        let data = reqwest::get(url).await?.json::<Value>().await?;

        let list = data.get("data")
            .and_then(|d| d.get("list"))
            .and_then(|l| l.as_array())
            .ok_or(anyhow!("json 转换失败"))?;

        for o in list {
            let ip = o.get("ip").and_then(|d| d.as_str()).ok_or(anyhow!("ip 转换失败"))?;
            let port = o.get("port").map(|d| d.to_string()).ok_or_else(|| anyhow!("port 转换失败"))?;
            proxies.push(ProxyBasic::new(ip, port.as_str()));
        }
    }

    Ok(proxies)
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_fetch() {
        let vec = super::fetch().await.unwrap();
        assert!(vec.len() > 0)
    }
}