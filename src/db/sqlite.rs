//! SQLite 存储模块
//!
//! 本模块实现了基于 SQLite 数据库的代理存储功能，
//! 包含代理基础信息的插入、代理质量数据的插入或更新、
//! 以及查询等操作。
//!
//! 通过 `SqliteStorage` 结构体封装数据库连接池，
//! 并实现了 `ProxyStorage` trait，
//! 方便上层调用统一接口操作代理数据。
//!
//! 该模块依赖于配置文件中的数据库连接信息和表名，
//! 且对表名进行校验以保证合法性。
//!
//! 适用于轻量级单机环境，
//! 通过 SQLite 实现高效的代理数据存储与管理。

use crate::db::manager::ProxyStorage;
use crate::model::{Proxy, ProxyBasic, APP_CONFIG};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tracing::info;
use crate::common::utils::validate_table_name;

#[derive(Debug)]
pub struct SqliteStorage {
    pool: Pool<Sqlite>,
}

impl SqliteStorage {
    pub async fn new() -> Result<Self> {

        let pool = SqlitePoolOptions::new()
            .max_connections(APP_CONFIG.db.max_connections)
            .connect(&APP_CONFIG.db.connection_string)
            .await?;

        let storage = Self { pool };
        storage.create_table().await?;
        info!("✅ SQLite 数据库连接成功");
        Ok(storage)
    }

    async fn create_table(&self) -> Result<()> {
        if !validate_table_name(&APP_CONFIG.db.table_name) {
            panic!("❌ 配置中的表名不合法：{}，请使用字母数字下划线，且不能以数字开头", APP_CONFIG.db.table_name);
        }

        let table = &APP_CONFIG.db.table_name;
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ip TEXT NOT NULL,
                port TEXT NOT NULL,
                speed REAL DEFAULT 0.0,
                success_rate REAL DEFAULT 0.0,
                stability REAL DEFAULT 0.0,
                score REAL DEFAULT 0.0,
                last_checked DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(ip, port)
            );
            "#,
            table
        ))
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl ProxyStorage for SqliteStorage {

    async fn insert_basic_proxy(&self, proxy: &ProxyBasic) -> Result<()> {
        sqlx::query(&format!(
            "INSERT OR IGNORE INTO {} (ip, port) VALUES (?, ?)",
            APP_CONFIG.db.table_name
        ))
            .bind(&proxy.ip)
            .bind(&proxy.port)
            .execute(&self.pool)
            .await?;
        info!("插入基础代理：{}:{}", proxy.ip, proxy.port);
        Ok(())
    }

    async fn upsert_quality_proxy(&self, proxy: &Proxy) -> Result<()> {
        sqlx::query(&format!(
            r#"
            INSERT INTO {} (ip, port, speed, success_rate, stability, score, last_checked)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(ip, port) DO UPDATE SET
                speed=excluded.speed,
                success_rate=excluded.success_rate,
                stability=excluded.stability,
                score=excluded.score,
                last_checked=excluded.last_checked
            "#,
            APP_CONFIG.db.table_name
        ))
            .bind(&proxy.ip)
            .bind(&proxy.port)
            .bind(proxy.speed)
            .bind(proxy.success_rate)
            .bind(proxy.stability)
            .bind(proxy.score)
            .bind(&proxy.last_checked)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_proxy_by_ip_port(&self, ip: &str, port: &str) -> Result<Option<Proxy>> {
        let proxy = sqlx::query_as::<_, Proxy>(&format!(
            "SELECT * FROM {} WHERE ip = ? AND port = ?",
            APP_CONFIG.db.table_name
        ))
            .bind(ip)
            .bind(port)
            .fetch_optional(&self.pool)
            .await?;
        Ok(proxy)
    }

    async fn list_all_proxies(&self) -> Result<Vec<Proxy>> {
        let proxies = sqlx::query_as::<_, Proxy>(&format!(
            "SELECT * FROM {} ORDER BY score DESC",
            APP_CONFIG.db.table_name
        ))
            .fetch_all(&self.pool)
            .await?;
        Ok(proxies)
    }

    async fn random_proxy(&self) -> Result<ProxyBasic> {
        let proxy= sqlx::query_as::<_, ProxyBasic>("SELECT * FROM proxies ORDER BY RANDOM() LIMIT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(proxy)
    }

    async fn remove_proxy(&self, ip: &str) -> Result<bool> {
        sqlx::query("DELETE FROM proxies WHERE ip = ?")
            .bind(ip)
            .execute(&self.pool)
            .await?;

        Ok(true)
    }
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
mod tests {
    use super::*;
    use crate::db::manager::ProxyStorage;
    use crate::model::{Proxy, ProxyBasic};
    use chrono::Utc;

    #[tokio::test]
    async fn test_create_table() {
        let storage = SqliteStorage::new().await.unwrap();
        let result = storage.create_table().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_insert_basic_proxy() {
        let storage = SqliteStorage::new().await.unwrap();
        let proxy = ProxyBasic::new("127.0.0.1", "1000");
        let result = storage.insert_basic_proxy(&proxy).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_quality_proxy() {
        let storage = SqliteStorage::new().await.unwrap();
        let proxy = Proxy {
            ip: "127.0.0.1".into(),
            port: "1001".into(),
            speed: Some(100.5),
            success_rate: Some(0.9),
            stability: Some(0.95),
            score: Some(85.0),
            last_checked: Some(Utc::now().naive_utc()),
        };

        let result = storage.upsert_quality_proxy(&proxy).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_proxy_by_ip_port() {
        let storage = SqliteStorage::new().await.unwrap();
        let proxy = ProxyBasic::new("127.0.0.1", "1002");

        storage.insert_basic_proxy(&proxy).await.unwrap();
        let found = storage.find_proxy_by_ip_port(&proxy.ip, &proxy.port).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().ip, proxy.ip);
    }

    #[tokio::test]
    async fn test_list_all_proxies() {
        let storage = SqliteStorage::new().await.unwrap();
        let result = storage.list_all_proxies().await;
        assert!(result.is_ok());
        let proxies = result.unwrap();
        assert!(proxies.len() > 0);
    }
}
