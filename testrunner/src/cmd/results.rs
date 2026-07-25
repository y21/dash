use rustc_hash::FxHashMap;

use crate::cmd::run::RunResult;

#[derive(Debug)]
pub struct ResultsMap(FxHashMap<String, RunResult>);

impl ResultsMap {
    pub const DEFAULT_CAPACITY: usize = 40000;

    pub fn new(cap: usize) -> Self {
        Self(FxHashMap::with_capacity_and_hasher(cap, Default::default()))
    }

    pub fn insert(&mut self, path: String, result: RunResult) {
        self.0.insert(path, result);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, RunResult)> {
        self.0.iter().map(|(k, v)| (k, *v))
    }

    pub fn get(&self, path: &str) -> Option<RunResult> {
        self.0.get(path).copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
