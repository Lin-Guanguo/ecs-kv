use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::Zset;

#[derive(Clone, Debug)]
pub struct Db {
    db: Arc<DbImpl>,
}

#[derive(Debug)]
struct DbImpl {
    kv: RwLock<HashMap<String, String>>,
    zset: RwLock<HashMap<String, RwLock<Zset>>>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            db: Arc::new(DbImpl {
                kv: RwLock::new(HashMap::new()),
                zset: RwLock::new(HashMap::new()),
            }),
        }
    }

    pub fn add(&self, k: String, v: String) {
        self.db.kv.write().unwrap().insert(k, v);
    }

    pub fn del(&self, k: &str) {
        self.db.kv.write().unwrap().remove(k);
        self.db.zset.write().unwrap().remove(k);
    }

    pub fn query(&self, k: &str) -> Option<String> {
        self.db.kv.read().unwrap().get(k).cloned()
    }

    pub fn zadd(&self, key: String, value: String, score: f64) {
        self.db
            .zset
            .write()
            .unwrap()
            .entry(key)
            .or_insert(RwLock::new(Zset::new()))
            .write()
            .unwrap()
            .add(value, score)
    }

    pub fn zremove(&self, key: &String, value: &String) {
        self.db
            .zset
            .read()
            .unwrap()
            .get(key)
            .map(|set| set.write().unwrap().remove(value));
    }

    pub fn zrange(&self, key: &String, min: f64, max: f64) -> Vec<String> {
        self.db
            .zset
            .read()
            .unwrap()
            .get(key)
            .map_or(Vec::new(), |set| set.read().unwrap().range(min, max))
    }
}
