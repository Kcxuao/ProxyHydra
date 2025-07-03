#![allow(dead_code)]
#![allow(unused_variables)]

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static APP_CONFIG: Lazy<AppConfig> =
    Lazy::new(|| AppConfig::load().expect("Failed to load configuration"));

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub verify: VerifyConfig,
    pub db: DbConfig,
    pub log: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbConfig {
    pub driver: String,
    pub connection_string: String,
    pub table_name: String,
    pub max_connections: u32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyConfig {
    pub semaphore: usize,
    pub timeout: u64,
    pub test_urls: Vec<String>,
    pub verify_level: u32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub console_levels: Vec<String>,
}


impl AppConfig {
    fn load() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("Config"))
            .build()?;
        let config = config.try_deserialize()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        println!("{:#?}", APP_CONFIG.verify.semaphore);
    }
}