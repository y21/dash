use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

use crate::gc::handle::Handle;
use crate::gc::interner::Symbol;
use crate::value::function::bound::BoundFunction;
use crate::value::promise::{Promise, PromiseState};
use crate::value::ValueContext;
use crate::PromiseAction;

use super::value::object::Object;
use super::value::Value;
use super::Vm;

use std::ptr::NonNull;

use crate::gc::trace::{Trace, TraceCtxt};

#[derive(Debug)]
pub struct LocalScopeList {
    head: Option<NonNull<ScopeData>>,
    // This contains all pointers.
    // Currently only exists so we can find all scopes at any time.
    // The `head` field does not contain scopes that are currently in use.
    // TODO: look into making this more efficient
    list: Vec<NonNull<ScopeData>>,
}

// NOTE: we can optimize this quite a bit, by deferring the work
// to `LocalScope::add_value` since we're creating a ton of scopes
// and not always adding values,
// but profiling and benchmarking has shown this to not bring
// any significant improvements.
// Maybe revisit later.
pub fn scope(vm: &mut Vm) -> LocalScope<'_> {
    match vm.scopes.head {
        Some(ptr) => {
            // We have an available scope we can use.
            let data = unsafe { ptr.as_ref() };
            let next = data.next;
            vm.scopes.head = next;
            LocalScope {
                _p: PhantomData,
                scope_data: ptr,
                vm,
            }
        }
        None => {
            // No scope available.
            let scope_data = ScopeData::new(None);
            vm.scopes.list.push(scope_data);
            LocalScope {
                _p: PhantomData,
                scope_data,
                vm,
            }
        }
    }
}

impl Drop for LocalScopeList {
    fn drop(&mut self) {
        let Self { mut head, list } = self;

        let mut drop_count = 0;
        while let Some(ptr) = head {
            let data = unsafe { ptr.as_ref() };
            head = data.next;
            // SAFETY: ptr was created using Box::into_raw
            unsafe { drop(Box::from_raw(ptr.as_ptr())) };
            drop_count += 1;
        }
        assert_eq!(
            drop_count,
            list.len(),
            "scope list corrupted: not all scopes were returned by the time LocalScopeList is dropped"
        );
    }
}

unsafe impl Trace for LocalScopeList {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        let Self { list, head: _ } = self;

        // We need to use the list instead of head,
        // because head may not contain all scopes
        // (more specifically, used scopes are removed from the link,
        // but remain in the list, so if we would use head,
        // we would miss those scopes!).
        for ptr in list {
            let data = unsafe { ptr.as_ref() };
            data.refs.trace(cx);
            data.strings.trace(cx);
        }
    }
}

impl LocalScopeList {
    pub fn new() -> Self {
        let node1 = ScopeData::new(None);
        let node2 = ScopeData::new(Some(node1));
        let node3 = ScopeData::new(Some(node2));
        let node4 = ScopeData::new(Some(node3));
        Self {
            head: Some(node4),
            list: vec![node1, node2, node3, node4],
        }
    }
}

impl Default for LocalScopeList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct ScopeData {
    refs: Vec<Handle>,
    strings: Vec<Symbol>,
    next: Option<NonNull<ScopeData>>,
}

impl ScopeData {
    pub fn new(next: Option<NonNull<Self>>) -> NonNull<Self> {
        NonNull::new(Box::into_raw(Box::new(Self {
            refs: Vec::with_capacity(4),
            strings: Vec::with_capacity(4),
            next,
        })))
        .unwrap()
    }
}

/// A handle to a [`LocalScope`], managed by [`LocalScopeList`].
#[derive(Debug)]
pub struct LocalScope<'vm> {
    vm: *mut Vm,
    scope_data: NonNull<ScopeData>,
    _p: PhantomData<&'vm mut Vm>,
}

impl<'vm> LocalScope<'vm> {
    fn scope_data_mut(&mut self) -> &mut ScopeData {
        unsafe { self.scope_data.as_mut() }
    }

    pub fn add_ref(&mut self, obj: Handle) {
        self.scope_data_mut().refs.push(obj);
    }

    pub fn add_value(&mut self, value: Value) {
        match value {
            Value::Object(o) => self.add_ref(o),
            Value::External(o) => {
                // Two things to add: the inner object, and the external itself
                // TODO: do we really need to add the inner object, considering that the inner will be traversed during tracing
                self.add_value(o.inner().clone());
                self.add_ref(o.as_gc_handle());
            }
            Value::String(s) => {
                self.scope_data_mut().strings.push(s.sym());
            }
            Value::Symbol(s) => {
                self.scope_data_mut().strings.push(s.sym());
            }
            _ => {}
        }
    }

    pub fn add_many(&mut self, v: &[Value]) {
        for val in v {
            self.add_value(val.clone());
        }
    }

    /// Registers an object and roots it.
    #[cfg_attr(feature = "stress_gc", track_caller)]
    pub fn register<O: Object + 'static>(&mut self, obj: O) -> Handle {
        #[allow(clippy::disallowed_methods)] // ok, we immediately root the handle
        let handle = self.deref_mut().register(obj);
        self.add_ref(handle.clone());
        handle
    }

    pub fn drive_promise(&mut self, action: PromiseAction, promise: &Promise, args: Vec<Value>) {
        let arg = args.first().unwrap_or_undefined();
        let mut state = promise.state().borrow_mut();

        if let PromiseState::Pending { resolve, reject } = &mut *state {
            let handlers = match action {
                PromiseAction::Resolve => mem::take(resolve),
                PromiseAction::Reject => mem::take(reject),
            };

            for handler in handlers {
                let bf = BoundFunction::new(self, handler, None, Some(args.clone()));
                let bf = self.register(bf);
                self.add_async_task(bf);
            }
        }

        *state = match action {
            PromiseAction::Resolve => PromiseState::Resolved(arg),
            PromiseAction::Reject => PromiseState::Rejected(arg),
        };
    }

    pub fn intern(&mut self, s: impl std::borrow::Borrow<str>) -> Symbol {
        let sym = self.interner.intern(s);
        self.scope_data_mut().strings.push(sym);
        sym
    }

    pub fn intern_usize(&mut self, n: usize) -> Symbol {
        let sym = self.interner.intern_usize(n);
        self.scope_data_mut().strings.push(sym);
        sym
    }

    pub fn intern_isize(&mut self, n: isize) -> Symbol {
        let sym = self.interner.intern_isize(n);
        self.scope_data_mut().strings.push(sym);
        sym
    }

    pub fn intern_char(&mut self, v: char) -> Symbol {
        let sym = self.interner.intern_char(v);
        self.scope_data_mut().strings.push(sym);
        sym
    }
}

// TODO: remove this Deref impl
// It's too prone to bugs due to similar methods
impl<'a> Deref for LocalScope<'a> {
    type Target = Vm;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.vm }
    }
}

impl<'a> DerefMut for LocalScope<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.vm }
    }
}

impl<'vm> Drop for LocalScope<'vm> {
    fn drop(&mut self) {
        let head = self.scopes.head;
        let data = self.scope_data_mut();
        let ScopeData { refs, strings, next } = data;
        refs.clear();
        strings.clear();
        *next = head;
        self.scopes.head = Some(self.scope_data);
    }
}

#[cfg(test)]
mod tests {
    use crate::value::string::JsString;
    use crate::Vm;

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
