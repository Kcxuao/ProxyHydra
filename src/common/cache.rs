use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use crate::model::Proxy;

pub struct GlobalCache<K, V> {
    store: Arc<RwLock<HashMap<K, V>>>,
}

impl<K: Eq + Hash + Clone, V: Clone> GlobalCache<K, V> {
    fn new() -> Self {
        GlobalCache {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    pub fn set(&self, key: K, value: V) {
        let mut store = self.store.write().unwrap();
        store.insert(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut store = self.store.write().unwrap();
        store.remove(key)
    }

    pub fn clear(&self) {
        let mut store = self.store.write().unwrap();
        store.clear();
    }
}

// 全局唯一缓存实例
pub static CACHE: Lazy<GlobalCache<&str, Vec<Proxy>>> = Lazy::new(|| GlobalCache::new());