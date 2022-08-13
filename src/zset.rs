use dashmap::DashMap;
use ordered_float::NotNan;
use std::collections::BTreeSet;
use std::ops::Bound::Included;
use std::sync::RwLock;

#[derive(Debug)]
pub struct Zset {
    kv: DashMap<String, NotNan<f64>>,
    vk: RwLock<BTreeSet<(NotNan<f64>, String)>>,
}

impl Zset {
    pub fn new() -> Self {
        Self {
            kv: DashMap::new(),
            vk: RwLock::new(BTreeSet::new()),
        }
    }

    pub fn add(&self, key: String, score: f64) {
        let score = NotNan::new(score).unwrap();
        self.kv.insert(key.clone(), score);
        self.vk.write().unwrap().insert((score, key));
    }

    pub fn remove(&self, k: &String) {
        let mut vk = self.vk.write().unwrap();
        self.kv.remove(k).map(|(k, v)| vk.remove(&(v, k)));
    }

    pub fn range(&self, min: f64, max: f64) -> Vec<String> {
        let min = NotNan::new(min).unwrap();
        let max = NotNan::new(max).unwrap();
        let vk = self.vk.read().unwrap();
        let range = vk.range((Included((min, "".into())), Included((max, "\x7f".into()))));
        range.map(|(_v, k)| k.clone()).collect()
    }
}
