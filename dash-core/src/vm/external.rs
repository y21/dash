use std::collections::HashMap;
use std::mem;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;

use super::local::LocalScope;
use super::value::object::Object;

#[derive(Debug)]
pub struct Externals(HashMap<*const LocalScope<'static>, Vec<Handle<dyn Object>>>);

unsafe impl Trace for Externals {
    fn trace(&self) {
        for ext in self.0.values() {
            ext.trace();
        }
    }
}

impl Externals {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub unsafe fn add<'a>(&mut self, sc: *const LocalScope<'a>, refs: Vec<Handle<dyn Object>>) {
        // lifetime transmute
        let sc = mem::transmute::<_, *const LocalScope<'static>>(sc);
        self.0.insert(sc, refs);
    }
}
