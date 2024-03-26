use rustc_hash::FxHashSet;

#[derive(Default)]
pub struct TaskIds {
    tasks: FxHashSet<u64>,
    generator: u64,
}

impl TaskIds {
    pub fn new() -> Self {
        Self {
            generator: 0,
            tasks: FxHashSet::default(),
        }
    }

    pub fn add(&mut self) -> u64 {
        let id = self.generator;
        self.generator += 1;
        self.tasks.insert(id);
        id
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }

    pub fn remove(&mut self, id: u64) -> bool {
        self.tasks.remove(&id)
    }
}
