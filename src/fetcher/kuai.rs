use crate::model::ProxyBasic;
use anyhow::Result;
use regex::Regex;
use tracing::info;

/// 从「快代理」网站分页抓取代理列表并解析为 `ProxyBasic` 实例集合。
///
/// 函数将遍历页码 `1..=3`，从每一页的 HTML 中提取嵌入的 `fpsList` JSON 数据，
/// 并解析为代理列表，最后合并返回。
///
/// # 数据源示例
/// 抓取目标地址形如：
/// - `https://www.kuaidaili.com/free/intr/1/`
/// - `https://www.kuaidaili.com/free/intr/2/`
///
/// 每页 HTML 中包含以下 JavaScript 片段：
/// ```html
/// <script>
///     const fpsList = [...]; // 包含代理的 JSON 数组
/// </script>
/// ```
///
/// # 返回
/// 成功时返回从 3 页中提取并解析出的所有 `ProxyBasic` 实例。
///
/// # 错误
/// - 网络请求失败时返回 `reqwest::Error`；  
/// - 页面中未找到匹配的正则表达式内容时，不会报错但会跳过该页；  
/// - 若 JSON 格式解析失败，则返回 `serde_json::Error`。
///
/// # 日志
/// 每页请求开始时输出类似：`正在请求第 1 页数据`
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

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_fetch() {
        let list = super::fetch().await.unwrap();
        assert!(list.len() > 0);
    }
}
