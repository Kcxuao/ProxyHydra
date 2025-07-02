use crate::fetcher::{bfbke, kuai};
use crate::model::ProxyBasic;
use anyhow::Result;

pub async fn fetch_all_sources() -> Result<Vec<ProxyBasic>> {
    let mut list = Vec::new();
    list.extend(bfbke::fetch().await?);
    list.extend(kuai::fetch().await?);
    Ok(list)
}
