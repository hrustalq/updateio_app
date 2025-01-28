use chrono::{DateTime, Duration, Utc};
use lru::LruCache;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

const CACHE_SIZE: usize = 1000;
const CACHE_TTL_MINUTES: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub needs_update: bool,
    pub checked_at: DateTime<Utc>,
}

impl CacheEntry {
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        now.signed_duration_since(self.checked_at) < Duration::minutes(CACHE_TTL_MINUTES)
    }
}

#[derive(Debug)]
pub struct UpdateCache {
    cache: Mutex<LruCache<u32, CacheEntry>>,
}

impl UpdateCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(CACHE_SIZE).unwrap())),
        }
    }

    pub fn get(&self, app_id: u32) -> Option<CacheEntry> {
        let mut cache = self.cache.lock();
        cache.get(&app_id).cloned().filter(|entry| entry.is_valid())
    }

    pub fn set(&self, app_id: u32, needs_update: bool) {
        let mut cache = self.cache.lock();
        let entry = CacheEntry {
            needs_update,
            checked_at: Utc::now(),
        };
        cache.put(app_id, entry);
    }

    pub fn invalidate(&self, app_id: u32) {
        let mut cache = self.cache.lock();
        cache.pop(&app_id);
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock();
        cache.clear();
    }
}

impl Default for UpdateCache {
    fn default() -> Self {
        Self::new()
    }
}
