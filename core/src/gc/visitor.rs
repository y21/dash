use std::collections::HashMap;

use crate::vm::value::Value;

use super::{DropContext, Handle};

/// A trait that implements GC related methods
pub trait GcVisitor {
    /// Deallocates every handle in reach
    unsafe fn drop_reachable(&mut self, cx: &mut DropContext);
}

impl<K> GcVisitor for HashMap<K, Handle<Value>> {
    unsafe fn drop_reachable(&mut self, cx: &mut DropContext) {
        for handle in self.values() {
            cx.drop(handle);
        }
    }
}

impl GcVisitor for Value {
    unsafe fn drop_reachable(&mut self, cx: &mut DropContext) {
        self.fields.drop_reachable(cx);

        if let Some(proto) = &self.proto {
            cx.drop(proto);
        }

        if let Some(constructor) = &self.constructor {
            cx.drop(constructor);
        }

        // todo: self.kind
    }
}
