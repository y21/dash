use rustc_hash::FxHashMap;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;

use super::local::LocalScope;
use super::value::object::Object;

#[derive(Debug, Default)]
pub struct Externals(FxHashMap<*const (), Vec<Handle<dyn Object>>>);

unsafe impl Trace for Externals {
    fn trace(&self) {
        for ext in self.0.values() {
            ext.trace();
        }
    }
}

impl Externals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extend_from_scope(&mut self, sc: *const LocalScope, mut refs: Vec<Handle<dyn Object>>) {
        self.0.entry(sc.cast()).or_insert(Vec::new()).append(&mut refs);
    }

    pub fn add_single(&mut self, sc: *const LocalScope, re: Handle<dyn Object>) {
        self.0.entry(sc.cast()).or_insert_with(Vec::new).push(re)
    }

    pub fn remove(&mut self, sc: *const LocalScope) -> Option<Vec<Handle<dyn Object>>> {
        self.0.remove(&sc.cast())
    }
}
