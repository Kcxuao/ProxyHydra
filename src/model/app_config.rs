use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub verify: VerifyConfig
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyConfig {
    pub semaphore: usize,
    pub timeout: u64,
    pub test_urls: Vec<String>,
}

pub static APP_CONFIG: Lazy<AppConfig> =
    Lazy::new(|| AppConfig::load().expect("Failed to load configuration"));

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