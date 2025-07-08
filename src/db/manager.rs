#![allow(dead_code)]
#![allow(unused_imports)]

//! 存储模块：支持多种数据库后端的代理数据读写接口。
//!
//! 支持的后端包括：SQLite、MySQL、PostgreSQL（按编译特性启用）
//! 提供统一的异步 trait [`ProxyStorage`]，便于通过 [`StorageBackend`] 多态调度。
use async_trait::async_trait;
use anyhow::Result;
#[cfg(feature = "mysql")]
use crate::db::mysql::MySqlStorage;
#[cfg(feature = "postgres")]
use crate::db::postgres::PgStorage;
use crate::db::sqlite::SqliteStorage;
use crate::model::{Proxy, ProxyBasic, APP_CONFIG};

/// 定义代理存储操作的通用异步接口。
///
/// 无论具体底层是 SQLite、MySQL 还是 PostgreSQL，
/// 都需实现此 trait，以实现代理数据的统一读写操作。
#[async_trait]
pub trait ProxyStorage: Send + Sync {
    /// 插入原始代理条目（无质量评估）。
    async fn insert_basic_proxy(&self, proxy: &ProxyBasic) -> Result<()>;

    /// 插入或更新包含质量信息的代理记录。
    async fn upsert_quality_proxy(&self, proxy: &Proxy) -> Result<()>;

    /// 根据 IP 和端口查找代理（用于去重或更新判断）。
    async fn find_proxy_by_ip_port(&self, ip: &str, port: &str) -> Result<Option<Proxy>>;

    /// 列出数据库中所有代理记录。
    async fn list_all_proxies(&self) -> Result<Vec<Proxy>>;

    async fn random_proxy(&self) -> Result<ProxyBasic>;
    async fn remove_proxy(&self, ip: &str) -> Result<bool>;
}

/// 数据库后端枚举，按启用特性动态支持多种数据库驱动。
///
/// 编译期通过 `features = ["sqlite", "mysql", "postgres"]` 控制可用性。
/// 运行时可通过配置项动态选择使用哪种后端。
#[derive(Debug)]
pub enum StorageBackend {
    /// SQLite 存储实现（轻量、文件型）
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteStorage),

    /// MySQL 存储实现（适用于大规模并发）
    #[cfg(feature = "mysql")]
    MySql(MySqlStorage),

    /// PostgreSQL 存储实现（事务强、扩展性好）
    #[cfg(feature = "postgres")]
    Postgres(PgStorage),
}

impl StorageBackend {
    /// 根据配置项创建对应的数据库后端实例。
    ///
    /// 依据 `APP_CONFIG.db.driver` 字符串值（如 "sqlite", "mysql", "postgres"），
    /// 创建相应的存储后端实例。
    ///
    /// # 返回
    /// 返回匹配的 [`StorageBackend`] 实例，或不支持的类型报错。
    pub async fn new() -> Result<Self> {
        match APP_CONFIG.db.driver.as_str() {
            #[cfg(feature = "sqlite")]
            "sqlite" => Ok(Self::Sqlite(SqliteStorage::new().await?)),
            #[cfg(feature = "mysql")]
            "mysql" => Ok(Self::MySql(MySqlStorage::new().await?)),
            #[cfg(feature = "postgres")]
            "postgres" => Ok(Self::Postgres(PgStorage::new().await?)),
            other => Err(anyhow::anyhow!("Unsupported DB type: {}", other)),
        }
    }
}

#[async_trait]
impl ProxyStorage for StorageBackend {
    async fn insert_basic_proxy(&self, proxy: &ProxyBasic) -> Result<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(s) => s.insert_basic_proxy(proxy).await,
            #[cfg(feature = "mysql")]
            Self::MySql(s) => s.insert_basic_proxy(proxy).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(s) => s.insert_basic_proxy(proxy).await,
        }
    }

    async fn upsert_quality_proxy(&self, proxy: &Proxy) -> Result<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(s) => s.upsert_quality_proxy(proxy).await,
            #[cfg(feature = "mysql")]
            Self::MySql(s) => s.upsert_quality_proxy(proxy).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(s) => s.upsert_quality_proxy(proxy).await,
        }
    }

    async fn find_proxy_by_ip_port(&self, ip: &str, port: &str) -> Result<Option<Proxy>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(s) => s.find_proxy_by_ip_port(ip, port).await,
            #[cfg(feature = "mysql")]
            Self::MySql(s) => s.find_proxy_by_ip_port(ip, port).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(s) => s.find_proxy_by_ip_port(ip, port).await,
        }
    }

    async fn list_all_proxies(&self) -> Result<Vec<Proxy>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(s) => s.list_all_proxies().await,
            #[cfg(feature = "mysql")]
            Self::MySql(s) => s.list_all_proxies().await,
            #[cfg(feature = "postgres")]
            Self::Postgres(s) => s.list_all_proxies().await,
        }
    }

    async fn random_proxy(&self) -> Result<ProxyBasic> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(s) => s.random_proxy().await,
            #[cfg(feature = "mysql")]
            Self::MySql(s) => s.random_proxy().await,
            #[cfg(feature = "postgres")]
            Self::Postgres(s) => s.random_proxy().await,
        }
    }

    async fn remove_proxy(&self, ip: &str) -> Result<bool> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(s) => s.remove_proxy(ip).await,
            #[cfg(feature = "mysql")]
            Self::MySql(s) => s.remove_proxy(ip).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(s) => s.remove_proxy(ip).await,
        }
    }
}
