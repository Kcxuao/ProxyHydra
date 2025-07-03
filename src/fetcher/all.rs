use crate::fetcher::{bfbke, kuai};
use crate::model::ProxyBasic;
use anyhow::Result;

/// 汇总所有代理来源的抓取结果，统一返回为 `ProxyBasic` 列表。
///
/// 该函数依次调用各个代理源模块的 `fetch()` 函数（如 `bfbke` 和 `kuai`），
/// 并将它们返回的代理合并为一个统一列表。
///
/// # 返回
/// 返回所有代理源合并后的 `Vec<ProxyBasic>`，其中每个元素表示一个原始代理条目。
///
/// # 错误
/// 如果任何一个源的抓取函数返回错误（如网络失败、格式异常），
/// 此函数也将立即返回相应的错误。
///
/// # 用例
/// 可作为统一的代理抓取入口，用于后续批量验证与质量评估流程。
pub async fn fetch_all_sources() -> Result<Vec<ProxyBasic>> {
    let mut list = Vec::new();
    list.extend(bfbke::fetch().await?);
    list.extend(kuai::fetch().await?);
    Ok(list)
}
