use once_cell::sync::OnceCell;
use crate::db::manager::StorageBackend;

/// 全局的可配置后端存储，支持 Sqlite/MySQL/Postgres
static GLOBAL_STORAGE: OnceCell<StorageBackend> = OnceCell::new();

/// 初始化全局存储，程序启动时调用一次即可
fn set_global_storage(backend: StorageBackend) {
    GLOBAL_STORAGE.set(backend).expect("Storage already initialized");
}

/// 在任意模块中使用此方法获取当前存储后端
pub fn get_storage() -> &'static StorageBackend {
    GLOBAL_STORAGE.get().expect("Storage not initialized")
}

pub async fn init() -> anyhow::Result<()> {
    let storage = StorageBackend::new().await?;
    set_global_storage(storage);
    Ok(())
}