use crate::db::insert;
use crate::proxy_server::Proxy;
use anyhow::Context;
use sqlx::FromRow;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
enum ApiError {
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    SQLError(#[from] sqlx::Error),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

mod db {
    use crate::proxy_server::Proxy;
    use sqlx::sqlite::{SqlitePoolOptions, SqliteQueryResult};
    use sqlx::{Error, Pool, Sqlite};
    use tokio::sync::OnceCell;
    use tracing::info;
    use tracing::log::error;

    static DB_POOL: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

    pub async fn init_db() {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite://proxy.db")
            .await
            .expect("Can't connect to DB");

        DB_POOL.set(pool).unwrap();
    }

    pub async fn insert(proxy: &Proxy) -> anyhow::Result<()> {
        let pool = DB_POOL.get().expect("DB_POOL not initialized");
        sqlx::query("insert into proxy(ip, port, sort) values ($1, $2, $3)")
            .bind(&proxy.ip)
            .bind(&proxy.port)
            .bind(&proxy.sort)
            .execute(pool)
            .await?;
        info!("插入成功：{:#?}", proxy.ip);
        Ok(())
    }

    pub async fn select_all() -> anyhow::Result<Vec<Proxy>> {
        let pool = DB_POOL.get().expect("DB_POOL not initialized");
        let list = sqlx::query_as::<_, Proxy>("select * from proxy order by sort desc")
            .fetch_all(pool)
            .await?;

        Ok(list)
    }
}

mod proxy_server {
    use anyhow::Context;
    use reqwest::ClientBuilder;
    use sqlx::FromRow;
    use std::time::Duration;

    #[derive(Debug,FromRow)]
    pub struct Proxy {
        pub id: i32,
        pub ip: String,
        pub port: i32,
        pub sort: i32
    }

    async fn get_proxy_info() {
        let url = "";
    }

    pub async fn valid_proxy(proxy: &Proxy) -> anyhow::Result<bool> {

        let valid_url = "https://cip.cc";

        let proxy = format!("http://{}:{}", proxy.ip, proxy.port);
        let req_proxy = reqwest::Proxy::all(proxy)?;
        let client = ClientBuilder::new().proxy(req_proxy).timeout(Duration::from_secs(2)).build()?;

        let response = client.get(valid_url).send()
            .await.context("请求超时")?
            .text()
            .await?;
        println!("{}", response);
        Ok(true)
    }

}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    db::init_db().await;

    let proxy = Proxy {
        id: 0,
        ip: "8.148.11.3".to_string(),
        port: 3128,
        sort: 0,
    };
    if let Err(e) = proxy_server::valid_proxy(&proxy).await {
        println!("{:#?}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{init_db, insert};
    use crate::proxy_server::Proxy;

    #[tokio::test]
    async fn test_insert() {
        init_db().await;

        let proxy = Proxy {
            id: 0,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            sort: 0,
        };

        match insert(&proxy).await.context("Can't insert proxy1") {
            Ok(_) => {}
            Err(e) => {
                error!("{:#?}", e);
            }
        };
    }

    #[tokio::test]
    async fn test_valid_proxy() {
        let proxy = Proxy {
            id: 0,
            ip: "8.148.11.3".to_string(),
            port: 3128,
            sort: 0,
        };
        proxy_server::valid_proxy(&proxy).await.unwrap();
    }
}
