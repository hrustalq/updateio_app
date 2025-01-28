use chrono::{DateTime, Duration, Utc};
use lru::LruCache;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::num::NonZeroUsize;

#[allow(dead_code)]
const CACHE_SIZE: usize = 1000;

#[allow(dead_code)]
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
#[allow(dead_code)]
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
}

impl Default for UpdateCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Cache<K: Hash + Eq, V> {
    cache: Mutex<LruCache<K, V>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TimedCacheEntry<T: Clone> {
    pub value: T,
    pub timestamp: DateTime<Utc>,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new(size: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(size).unwrap())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let mut cache = self.cache.lock();
        cache.get(key).cloned()
    }

    pub fn set(&self, key: K, value: V) {
        let mut cache = self.cache.lock();
        cache.put(key, value);
    }

    pub fn invalidate(&self, key: &K) {
        let mut cache = self.cache.lock();
        cache.pop(key);
    }
}

impl<T: Clone> TimedCacheEntry<T> {
    pub fn new(value: T, ttl_minutes: i64) -> Self {
        Self {
            value,
            timestamp: Utc::now() + Duration::minutes(ttl_minutes),
        }
    }
}
