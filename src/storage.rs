#![allow(dead_code)]
#![allow(unused_variables)]

use crate::model::Proxy;
use anyhow::Result;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use tokio::sync::OnceCell;
use tracing::info;

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
    let res = sqlx::query(
        "CREATE TABLE IF NOT EXISTS proxy (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ip TEXT NOT NULL,
        port TEXT NOT NULL,
        speed REAL,             -- 小数类型，支持保留两位小数
        success_rate REAL,
        stability REAL,
        score REAL,             -- 小数类型, 0-1 之间
        last_checked TEXT,
        UNIQUE(ip, port)
    );",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_proxy_by_ip_port(ip: &str, port: &str) -> Result<Option<Proxy>> {
    let pool = DB_POOL.get().expect("DB_POOL not initialized");
    let proxy = sqlx::query_as::<_, Proxy>("SELECT * FROM proxy WHERE ip = ? AND port = ?")
        .bind(ip)
        .bind(port)
        .fetch_optional(pool)
        .await?;
    Ok(proxy)
}

pub async fn insert_basic_proxy(proxy: &Proxy) -> Result<()> {
    let pool = DB_POOL.get().expect("DB_POOL not initialized");
    sqlx::query("INSERT OR IGNORE INTO proxy(ip, port) VALUES (?, ?)")
        .bind(&proxy.ip)
        .bind(&proxy.port)
        .execute(pool)
        .await?;
    info!("成功插入代理：{}:{}", proxy.ip, proxy.port);
    Ok(())
}

pub async fn upsert_quality_proxy(proxy: &Proxy) -> Result<()> {
    let pool = DB_POOL.get().expect("DB_POOL not initialized");
    sqlx::query(
        "INSERT OR REPLACE INTO proxy 
            (ip, port, speed, success_rate, stability, score, last_checked)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
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

    #[tokio::test]
    async fn test_insert_and_find_proxy() {
        init().await.unwrap();
        let a = list_all_proxies().await.unwrap();
        println!("{:#?}", a);
    }
}
