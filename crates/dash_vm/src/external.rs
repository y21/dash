use rustc_hash::FxHashMap;

use crate::gc::handle::Handle;
use crate::gc::trace::{Trace, TraceCtxt};

use super::localscope::LocalScope;

#[derive(Debug, Default)]
pub struct Externals(FxHashMap<*const (), Vec<Handle>>);

unsafe impl Trace for Externals {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        for ext in self.0.values() {
            ext.trace(cx);
        }
    }
}

impl Externals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extend_from_scope(&mut self, sc: *const LocalScope, mut refs: Vec<Handle>) {
        self.0.entry(sc.cast()).or_default().append(&mut refs);
    }

    pub fn add_single(&mut self, sc: *const LocalScope, re: Handle) {
        self.0.entry(sc.cast()).or_default().push(re)
    }

    pub fn remove(&mut self, sc: *const LocalScope) -> Option<Vec<Handle>> {
        self.0.remove(&sc.cast())
    }
}
