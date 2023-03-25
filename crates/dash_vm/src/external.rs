use std::collections::HashMap;

use crate::gc2::handle::Handle;
use crate::gc2::trace::Trace;

use super::local::LocalScope;
use super::value::object::Object;

#[derive(Debug, Default)]
pub struct Externals(HashMap<*const (), Vec<Handle<dyn Object>>>);

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

    pub fn add(&mut self, sc: *const LocalScope, refs: Vec<Handle<dyn Object>>) {
        self.0.insert(sc.cast(), refs);
    }

    pub fn add_single(&mut self, sc: *const LocalScope, re: Handle<dyn Object>) {
        self.0.entry(sc.cast()).or_insert_with(Vec::new).push(re)
    }

    pub fn remove(&mut self, sc: *const LocalScope) -> Option<Vec<Handle<dyn Object>>> {
        self.0.remove(&sc.cast())
    }
}
