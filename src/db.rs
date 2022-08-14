use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{zset::ScoreValue, Zset};
use dashmap::DashMap;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Db {
    db: Arc<DbImpl>,
}

#[derive(Debug)]
struct DbImpl {
    kv: [DashMap<String, DbValue>; 32],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
enum DbValue {
    Text(String),
    Zset(Arc<Zset>),
}

impl Db {
    pub fn new() -> Self {
        Self {
            db: Arc::new(DbImpl {
                kv: [
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                    DashMap::new(),
                ],
            }),
        }
    }

    fn hash(k: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        k.hash(&mut hasher);
        hasher.finish() as usize % 32
    }

    pub fn add(&self, kv: KeyValue) {
        self.db.kv[Self::hash(&kv.key)].insert(kv.key, DbValue::Text(kv.value));
    }

    pub fn batch(&self, keys: Vec<KeyValue>) {
        keys.into_iter().for_each(|kv| self.add(kv))
    }

    pub fn del(&self, key: &str) {
        self.db.kv[Self::hash(&key)].remove(key);
    }

    pub fn query(&self, key: &str) -> Option<String> {
        self.db.kv[Self::hash(&key)]
            .get(key)
            .and_then(|entry| match entry.value() {
                DbValue::Text(v) => Some(v.clone()),
                DbValue::Zset(_) => None,
            })
    }

    pub fn list(&self, keys: Vec<String>) -> Vec<KeyValue> {
        keys.into_iter()
            .filter_map(|key| self.query(&key).map(|value| KeyValue { key, value }))
            .collect()
    }

    pub fn zadd(&self, key: String, value: String, score: f64) {
        let h = Self::hash(&key);
        let cur_zset = self.db.kv[h].get(&key).and_then(|v| match v.value() {
            DbValue::Text(_) => None,
            DbValue::Zset(zset) => Some(zset.clone()),
        });
        let zset = match cur_zset {
            Some(zset) => zset,
            None => {
                let z = Arc::new(Zset::new());
                self.db.kv[h].insert(key, DbValue::Zset(z.clone()));
                z
            }
        };
        zset.add(value, score)
    }

    pub fn zremove(&self, key: &str, value: &str) {
        let h = Self::hash(&key);
        let cur_zset = self.db.kv[h].get(key).and_then(|v| match v.value() {
            DbValue::Text(_) => None,
            DbValue::Zset(zset) => Some(zset.clone()),
        });
        cur_zset.map(|v| v.remove(value));
    }

    pub fn zrange(&self, key: &str, min: f64, max: f64) -> Vec<ScoreValue> {
        let h = Self::hash(&key);
        let cur_zset = self.db.kv[h].get(key).and_then(|v| match v.value() {
            DbValue::Text(_) => None,
            DbValue::Zset(zset) => Some(zset.clone()),
        });
        cur_zset.map_or_else(|| Vec::new(), |v| v.range(min, max))
    }
}
