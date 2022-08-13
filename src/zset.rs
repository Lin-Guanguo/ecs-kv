use ordered_float::NotNan;
use std::collections::{BTreeSet, HashMap};
use std::ops::Bound::Included;

#[derive(Debug)]
pub struct Zset {
    kv: HashMap<String, NotNan<f64>>,
    vk: BTreeSet<(NotNan<f64>, String)>,
}

impl Zset {
    pub fn new() -> Self {
        Self {
            kv: HashMap::new(),
            vk: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, k: String, v: f64) {
        let v = NotNan::new(v).unwrap();
        self.kv.insert(k.clone(), v);
        self.vk.insert((v, k));
    }

    pub fn remove(&mut self, k: &String) {
        self.kv
            .remove_entry(k)
            .map(|(k, v)| self.vk.remove(&(v, k)));
    }

    pub fn range(&self, min: f64, max: f64) -> Vec<String> {
        let min = NotNan::new(min).unwrap();
        let max = NotNan::new(max).unwrap();
        let range = self
            .vk
            .range((Included((min, "".into())), Included((max, "\x7f".into()))));
        range.map(|(_v, k)| k.clone()).collect()
    }
}
