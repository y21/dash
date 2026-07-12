use std::mem;
use std::ops::{Deref, DerefMut};

use dash_middle::interner::Symbol;

use crate::gc::ObjectId;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::value::function::args::CallArgs;
use crate::value::function::bound::BoundFunction;
use crate::value::object::Object;
use crate::value::promise::{Promise, PromiseState};
use crate::value::{Unpack, Value, ValueContext, ValueKind};
use crate::{PromiseAction, Vm};

pub enum ShadowRoot {
    Object(ObjectId),
    Symbol(Symbol),
}

pub trait TryShadowRoot {
    fn try_into_shadow_root(self) -> Option<ShadowRoot>;
}

impl TryShadowRoot for ObjectId {
    fn try_into_shadow_root(self) -> Option<ShadowRoot> {
        Some(ShadowRoot::Object(self))
    }
}

impl TryShadowRoot for Symbol {
    fn try_into_shadow_root(self) -> Option<ShadowRoot> {
        Some(ShadowRoot::Symbol(self))
    }
}

impl TryShadowRoot for Value {
    fn try_into_shadow_root(self) -> Option<ShadowRoot> {
        match self.unpack() {
            ValueKind::Object(id) => Some(ShadowRoot::Object(id)),
            ValueKind::External(ext) => Some(ShadowRoot::Object(ext.id())),
            ValueKind::Symbol(sym) => Some(ShadowRoot::Symbol(sym.sym())),
            ValueKind::String(str) => Some(ShadowRoot::Symbol(str.sym())),
            _ => None,
        }
    }
}

impl<T: Copy + TryShadowRoot> TryShadowRoot for &T {
    fn try_into_shadow_root(self) -> Option<ShadowRoot> {
        (*self).try_into_shadow_root()
    }
}

unsafe impl Trace for ShadowRoot {
    fn trace(&self, ctxt: &mut TraceCtxt) {
        match self {
            ShadowRoot::Object(id) => id.trace(ctxt),
            ShadowRoot::Symbol(sym) => sym.trace(ctxt),
        }
    }
}

#[derive(Debug)]
pub struct LocalScope<'a> {
    vm: &'a mut Vm,
    /// The length of the vm's shadow_roots stack when this scope was created.
    stack_len: usize,
}

impl<'a> LocalScope<'a> {
    pub fn new(vm: &'a mut Vm) -> Self {
        let stack_len = vm.shadow_roots.len();
        Self { vm, stack_len }
    }

    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    pub fn add(&mut self, root: impl TryShadowRoot) {
        if let Some(shadow_root) = root.try_into_shadow_root() {
            self.vm.shadow_roots.push(shadow_root);
        }
    }

    pub fn add_many(&mut self, roots: impl IntoIterator<Item = impl TryShadowRoot>) {
        roots.into_iter().for_each(|root| self.add(root));
    }

    pub fn drive_promise(&mut self, action: PromiseAction, promise: &Promise, promise_id: ObjectId, args: CallArgs) {
        let arg = args.first().unwrap_or_undefined();
        let mut state = promise.state().borrow_mut();

        let mut has_handler = false;

        if let PromiseState::Pending { resolve, reject } = &mut *state {
            let handlers = match action {
                PromiseAction::Resolve => mem::take(resolve),
                PromiseAction::Reject => mem::take(reject),
            };

            has_handler = !handlers.is_empty();

            for handler in handlers {
                let bf = BoundFunction::new(self, handler, None, args.clone());
                let bf = self.register(bf);
                self.add_async_task(bf);
            }
        }

        *state = match action {
            PromiseAction::Resolve => PromiseState::Resolved(arg),
            PromiseAction::Reject => {
                self.rejected_promises.insert(promise_id);
                PromiseState::Rejected {
                    value: arg,
                    caught: has_handler,
                }
            }
        };
    }

    pub fn intern(&mut self, s: impl std::borrow::Borrow<str>) -> Symbol {
        let sym = self.vm.interner.intern(s);
        self.vm.shadow_roots.push(ShadowRoot::Symbol(sym));
        sym
    }

    pub fn intern_usize(&mut self, n: usize) -> Symbol {
        let sym = self.vm.interner.intern_usize(n);
        self.vm.shadow_roots.push(ShadowRoot::Symbol(sym));
        sym
    }

    pub fn intern_isize(&mut self, n: isize) -> Symbol {
        let sym = self.vm.interner.intern_isize(n);
        self.vm.shadow_roots.push(ShadowRoot::Symbol(sym));
        sym
    }

    pub fn intern_char(&mut self, v: char) -> Symbol {
        let sym = self.vm.interner.intern_char(v);
        self.vm.shadow_roots.push(ShadowRoot::Symbol(sym));
        sym
    }

    pub fn register<O: Object + 'static>(&mut self, obj: O) -> ObjectId {
        let id = self.vm.alloc.alloc_object(obj);
        self.vm.shadow_roots.push(ShadowRoot::Object(id));
        id
    }

    pub fn mk_promise(&mut self) -> ObjectId {
        let promise = Promise::new(self);
        self.register(promise)
    }
}

impl Drop for LocalScope<'_> {
    fn drop(&mut self) {
        self.vm.shadow_roots.truncate(self.stack_len);
    }
}

impl<'a> Deref for LocalScope<'a> {
    type Target = Vm;
    fn deref(&self) -> &Self::Target {
        self.vm
    }
}

impl<'a> DerefMut for LocalScope<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vm
    }
}

#[cfg(test)]
mod tests {
    use crate::Vm;
    use crate::value::string::JsString;

    #[test]
    fn it_works() {
        let mut vm = Vm::new(Default::default());
        let mut scope = vm.scope();
        for _ in 0..20 {
            let val = scope.intern("test");
            scope.register(JsString::from(val));
        }
    }

    #[test]
    fn multiple_scopes() {
        let mut vm = Vm::new(Default::default());
        let mut scope = vm.scope();
        let mut scope1 = scope.scope();
        let mut scope2 = scope1.scope();
        let mut scope3 = scope2.scope();
        let mut scope4 = scope3.scope();
        let mut scope5 = scope4.scope();
        let k = scope5.intern("bar");
        scope5.register(JsString::from(k));
        let mut scope6 = scope5.scope();
        let mut scope7 = scope6.scope();
        let mut scope8 = scope7.scope();
        let k = scope8.intern("foo");
        scope8.register(JsString::from(k));
        let mut scope9 = scope8.scope();
        let k = scope9.intern("test");
        scope9.register(JsString::from(k));
    }
}
