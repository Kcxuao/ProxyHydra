use std::fs::{create_dir_all, File};
use std::path::Path;
use tracing::level_filters::LevelFilter;
use tracing::{Level, Metadata};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};
use crate::common::utils::parse_level;
use crate::model::APP_CONFIG;

pub fn init_logging() -> anyhow::Result<()> {
    let log_dir = Path::new("logs");
    create_dir_all(log_dir)?;

    let allowed_levels: Vec<Level> = APP_CONFIG
        .log.console_levels
        .iter()
        .filter_map(|lvl_str| parse_level(lvl_str))
        .collect();

    // 2. 文件日志（接收所有级别）
    let file = File::create(log_dir.join("all.log"))?;
    let file_layer = fmt::layer()
        .with_writer(file)
        .with_ansi(false)
        .with_filter(LevelFilter::DEBUG);

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_filter(filter_fn(move |metadata: &Metadata| {
            allowed_levels.contains(&metadata.level())
        }));

    // 4. 组合所有层
    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    Ok(())
}