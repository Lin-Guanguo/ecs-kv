use std::sync::Arc;

use crate::{zset::ScoreValue, Zset};
use dashmap::DashMap;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Db {
    db: Arc<DbImpl>,
}

#[derive(Debug)]
struct DbImpl {
    kv: DashMap<String, DbValue>,
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
            db: Arc::new(DbImpl { kv: DashMap::new() }),
        }
    }

    pub fn add(&self, key: String, value: String) {
        self.db.kv.insert(key, DbValue::Text(value));
    }

    pub fn del(&self, key: &str) {
        self.db.kv.remove(key);
    }

    pub fn query(&self, key: &str) -> Option<String> {
        self.db.kv.get(key).and_then(|entry| match entry.value() {
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
        let cur_zset = self.db.kv.get(&key).and_then(|v| match v.value() {
            DbValue::Text(_) => None,
            DbValue::Zset(zset) => Some(zset.clone()),
        });
        let zset = match cur_zset {
            Some(zset) => zset,
            None => {
                let z = Arc::new(Zset::new());
                self.db.kv.insert(key, DbValue::Zset(z.clone()));
                z
            }
        };
        zset.add(value, score)
    }

    pub fn zremove(&self, key: &str, value: &String) {
        let cur_zset = self.db.kv.get(key).and_then(|v| match v.value() {
            DbValue::Text(_) => None,
            DbValue::Zset(zset) => Some(zset.clone()),
        });
        cur_zset.map(|v| v.remove(value));
    }

    pub fn zrange(&self, key: &String, min: f64, max: f64) -> Vec<ScoreValue> {
        let cur_zset = self.db.kv.get(key).and_then(|v| match v.value() {
            DbValue::Text(_) => None,
            DbValue::Zset(zset) => Some(zset.clone()),
        });
        cur_zset.map_or_else(|| Vec::new(), |v| v.range(min, max))
    }
}
