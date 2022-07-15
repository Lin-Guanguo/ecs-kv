use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct Db {
    db: Arc<DbImpl>,
}

#[derive(Debug)]
struct DbImpl {
    kv: Mutex<HashMap<String, String>>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            db: Arc::new(DbImpl {
                kv: Mutex::new(HashMap::new()),
            }),
        }
    }

    fn db(&self) -> std::sync::MutexGuard<HashMap<String, String>> {
        self.db.kv.lock().unwrap()
    }

    pub fn add(&self, k: String, v: String) {
        self.db().insert(k, v);
    }

    pub fn del(&self, k: &str) {
        self.db().remove(k);
    }

    pub fn query(&self, k: &str) -> Option<String> {
        self.db().get(k).cloned()
    }
}
