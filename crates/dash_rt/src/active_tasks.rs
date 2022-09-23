use std::collections::HashSet;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

pub struct TaskIds {
    tasks: Mutex<HashSet<u64>>,
    generator: AtomicU64,
}

impl TaskIds {
    pub fn new() -> Self {
        Self {
            generator: AtomicU64::new(0),
            tasks: Mutex::new(HashSet::new()),
        }
    }

    pub fn add(&self) -> u64 {
        let id = self.generator.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.tasks.lock().unwrap().insert(id);
        id
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub fn remove(&self, id: u64) -> bool {
        self.tasks.lock().unwrap().remove(&id)
    }
}
