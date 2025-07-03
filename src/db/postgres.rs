#![allow(dead_code)]
#![allow(unused_imports)]

//! PostgreSQL 存储模块：基于 [`ProxyStorage`] trait 实现代理数据的持久化功能。
//!
//! 支持表结构初始化、基础代理插入、代理质量信息更新、按 IP/端口查找和列表排序查询。
//! 使用 `sqlx` 异步驱动，需启用 `postgres` 编译特性。
use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "postgres")]
use sqlx::{PgPool, postgres::PgPoolOptions};
use crate::model::{APP_CONFIG, Proxy, ProxyBasic};
use crate::db::manager::ProxyStorage;
use tracing::info;
use crate::common::utils::validate_table_name;

/// PostgreSQL 数据库存储实现，持有一个连接池 [`PgPool`]。
///
/// 实现了 [`ProxyStorage`] trait，用于插入、更新、查询代理数据。
#[cfg(feature = "postgres")]
#[derive(Debug)]
pub struct PgStorage {
    pool: PgPool,
}
#[cfg(feature = "postgres")]
impl PgStorage {
    /// 创建一个 PostgreSQL 存储实例并自动初始化数据表结构。
    ///
    /// # 返回
    /// 返回 [`PgStorage`] 实例，如果连接失败或建表失败则返回错误。
    pub async fn new() -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(APP_CONFIG.db.max_connections)
            .connect(&APP_CONFIG.db.connection_string)
            .await?;

        let storage = Self { pool };
        storage.create_table().await?;
        info!("✅ PostgresSQL 数据库连接成功");
        Ok(storage)
    }

    /// 创建用于存储代理信息的 PostgreSQL 表（如果不存在）。
    ///
    /// 表名从配置项 [`APP_CONFIG.db.table_name`] 读取，
    /// 并设置 `(ip, port)` 组合唯一约束。
    async fn create_table(&self) -> Result<()> {
        if !validate_table_name(&APP_CONFIG.db.table_name) {
            panic!("❌ 配置中的表名不合法：{}，请使用字母数字下划线，且不能以数字开头", APP_CONFIG.db.table_name);
        }
        
        let table = &APP_CONFIG.db.table_name;
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id SERIAL PRIMARY KEY,
                ip TEXT NOT NULL,
                port TEXT NOT NULL,
                speed DOUBLE PRECISION DEFAULT 0.0,
                success_rate DOUBLE PRECISION DEFAULT 0.0,
                stability DOUBLE PRECISION DEFAULT 0.0,
                score DOUBLE PRECISION DEFAULT 0.0,
                last_checked TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
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

#[cfg(feature = "postgres")]
#[async_trait]
impl ProxyStorage for PgStorage {
    async fn insert_basic_proxy(&self, proxy: &ProxyBasic) -> Result<()> {
        let table = &APP_CONFIG.db.table_name;
        let sql = format!("INSERT INTO {} (ip, port) VALUES ($1, $2) ON CONFLICT DO NOTHING", table);
        sqlx::query(&sql)
            .bind(&proxy.ip)
            .bind(&proxy.port)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn upsert_quality_proxy(&self, proxy: &Proxy) -> Result<()> {
        let table = &APP_CONFIG.db.table_name;
        let sql = format!(
            r#"
            INSERT INTO {} (ip, port, speed, success_rate, stability, score, last_checked)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(ip, port) DO UPDATE SET
                speed=EXCLUDED.speed,
                success_rate=EXCLUDED.success_rate,
                stability=EXCLUDED.stability,
                score=EXCLUDED.score,
                last_checked=EXCLUDED.last_checked
            "#,
            table
        );
        sqlx::query(&sql)
            .bind(&proxy.ip)
            .bind(&proxy.port)
            .bind(&proxy.speed)
            .bind(&proxy.success_rate)
            .bind(&proxy.stability)
            .bind(&proxy.score)
            .bind(&proxy.last_checked)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_proxy_by_ip_port(&self, ip: &str, port: &str) -> Result<Option<Proxy>> {
        let table = &APP_CONFIG.db.table_name;
        let sql = format!("SELECT * FROM {} WHERE ip = $1 AND port = $2", table);
        let result = sqlx::query_as::<_, Proxy>(&sql)
            .bind(ip)
            .bind(port)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    async fn list_all_proxies(&self) -> Result<Vec<Proxy>> {
        let table = &APP_CONFIG.db.table_name;
        let sql = format!("SELECT * FROM {} ORDER BY score DESC", table);
        let proxies = sqlx::query_as::<_, Proxy>(&sql)
            .fetch_all(&self.pool)
            .await?;
        Ok(proxies)
    }
}


#[cfg(test)]
#[cfg(feature = "postgres")]
mod tests {
    use chrono::{NaiveDateTime, Utc};
    use sqlx::{Encode, Postgres, Type};
    use super::*;
    use crate::model::{Proxy, ProxyBasic};
    use crate::db::manager::ProxyStorage;

    #[tokio::test]
    async fn test_create_table() {
        let storage = PgStorage::new().await.unwrap();
        let result = storage.create_table().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_insert_basic_proxy() {
        let storage = PgStorage::new().await.unwrap();
        let proxy = ProxyBasic::new("127.0.0.1", "1000");
        let result = storage.insert_basic_proxy(&proxy).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_quality_proxy() {
        let storage = PgStorage::new().await.unwrap();
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
        let storage = PgStorage::new().await.unwrap();
        let proxy = ProxyBasic::new("127.0.0.1", "1002");

        storage.insert_basic_proxy(&proxy).await.unwrap();
        let found = storage.find_proxy_by_ip_port(&proxy.ip, &proxy.port).await.unwrap();
        
        assert!(found.is_some());
        assert_eq!(found.unwrap().ip, proxy.ip);
    }

    #[tokio::test]
    async fn test_list_all_proxies() {
        let storage = PgStorage::new().await.unwrap();
        let result = storage.list_all_proxies().await;
        assert!(result.is_ok());
        let proxies = result.unwrap();
        assert!(!proxies.is_empty());
    }
}
