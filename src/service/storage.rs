#![allow(dead_code)]
#![allow(unused_variables)]

use anyhow::Result;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use tokio::sync::OnceCell;
use tracing::info;
use crate::model::{Proxy, ProxyBasic};

static DB_POOL: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

pub async fn init() -> Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://proxy.db")
        .await?;
    DB_POOL.set(pool)?;

    info!("数据库连接成功");
    create_table().await?;
    Ok(())
}

async fn create_table() -> Result<()> {
    let pool = DB_POOL.get().expect("DB_POOL not initialized");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS proxy (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ip TEXT NOT NULL,
            port TEXT NOT NULL,
            speed REAL,
            success_rate REAL,
            stability REAL,
            score REAL,
            last_checked TEXT,
            UNIQUE(ip, port)
        );
        "#
    )
        .execute(pool)
        .await?;

    Ok(())
}

/// 根据 IP 和端口查找代理，返回完整的 Proxy 结构
pub async fn find_proxy_by_ip_port(ip: &str, port: &str) -> Result<Option<Proxy>> {
    let pool = DB_POOL.get().expect("DB_POOL not initialized");
    let proxy = sqlx::query_as::<_, Proxy>(
        "SELECT * FROM proxy WHERE ip = ? AND port = ?"
    )
        .bind(ip)
        .bind(port)
        .fetch_optional(pool)
        .await?;
    Ok(proxy)
}

/// 插入基础代理信息，只插入 IP 和端口
pub async fn insert_basic_proxy(proxy_basic: &ProxyBasic) -> Result<()> {
    let pool = DB_POOL.get().expect("DB_POOL not initialized");
    sqlx::query("INSERT OR IGNORE INTO proxy(ip, port) VALUES (?, ?)")
        .bind(&proxy_basic.ip)
        .bind(&proxy_basic.port)
        .execute(pool)
        .await?;
    info!("成功插入代理：{}:{}", proxy_basic.ip, proxy_basic.port);
    Ok(())
}

/// 插入或更新代理的质量信息（如果已存在则覆盖）
pub async fn upsert_quality_proxy(proxy: &Proxy) -> Result<()> {
    let pool = DB_POOL.get().expect("DB_POOL not initialized");
    sqlx::query(
        r#"
        INSERT INTO proxy (ip, port, speed, success_rate, stability, score, last_checked)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(ip, port) DO UPDATE SET
            speed=excluded.speed,
            success_rate=excluded.success_rate,
            stability=excluded.stability,
            score=excluded.score,
            last_checked=excluded.last_checked
        "#
    )
        .bind(&proxy.ip)
        .bind(&proxy.port)
        .bind(&proxy.speed)
        .bind(&proxy.success_rate)
        .bind(&proxy.stability)
        .bind(&proxy.score)
        .bind(&proxy.last_checked)
        .execute(pool)
        .await?;
    Ok(())
}

/// 列出所有代理，按分数降序排序
pub async fn list_all_proxies() -> Result<Vec<Proxy>> {
    let pool = DB_POOL.get().expect("DB_POOL not initialized");
    let rows = sqlx::query_as::<_, Proxy>("SELECT * FROM proxy ORDER BY score DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProxyBasic;

    #[tokio::test]
    async fn test_insert_and_find_proxy() {
        init().await.unwrap();

        let proxy_basic = ProxyBasic::new("127.0.0.1".to_string(), "8080".to_string());
        insert_basic_proxy(&proxy_basic).await.unwrap();

        let proxy_opt = find_proxy_by_ip_port(&proxy_basic.ip, &proxy_basic.port).await.unwrap();
        println!("{:?}", proxy_opt);
    }
}
