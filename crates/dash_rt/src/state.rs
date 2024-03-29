use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dash_vm::gc::handle::Handle;
use dash_vm::gc::trace::Trace;
use dash_vm::Vm;
use rustc_hash::FxHashMap;

use crate::active_tasks::TaskIds;
use crate::event::EventSender;
use crate::module::ModuleLoader;
use crate::typemap::TypeMap;

pub struct State {
    rt: tokio::runtime::Handle,
    tx: EventSender,
    root_module: Rc<RefCell<Option<Box<dyn ModuleLoader>>>>,
    pub tasks: TaskIds,
    promises: FxHashMap<u64, Handle>,
    pub store: TypeMap,
}
unsafe impl Trace for State {
    fn trace(&self, cx: &mut dash_vm::gc::trace::TraceCtxt<'_>) {
        let Self {
            rt: _,
            tx: _,
            root_module: _,
            tasks: _,
            promises,
            store,
        } = self;
        promises.trace(cx);
        store.trace(cx);
    }
}

impl State {
    pub fn new(rt: tokio::runtime::Handle, tx: EventSender) -> Self {
        Self {
            rt,
            tx,
            root_module: Rc::new(RefCell::new(None)),
            tasks: TaskIds::new(),
            promises: FxHashMap::default(),
            store: TypeMap::default(),
        }
    }

    pub(crate) fn set_root_module(&self, module: Box<dyn ModuleLoader>) {
        self.root_module.replace(Some(module));
    }

    pub fn root_module(&self) -> &Rc<RefCell<Option<Box<dyn ModuleLoader>>>> {
        &self.root_module
    }

    pub fn needs_event_loop(&self) -> bool {
        self.tasks.has_tasks() || !self.promises.is_empty()
    }

    pub fn try_from_vm_mut(vm: &mut Vm) -> Option<&mut Self> {
        vm.params_mut().state_mut()
    }

    pub fn try_from_vm(vm: &Vm) -> Option<&Self> {
        vm.params().state()
    }

    /// Same as try_from_vm, but panics if state failed to downcast (i.e. Vm was not created by a runtime, or was changed at runtime)
    ///
    /// Usually it is a programmer error if downcasting fails, so this method is preferred
    pub fn from_vm_mut(vm: &mut Vm) -> &mut Self {
        Self::try_from_vm_mut(vm).unwrap()
    }

    pub fn from_vm(vm: &Vm) -> &Self {
        Self::try_from_vm(vm).unwrap()
    }

    pub fn event_sender(&self) -> EventSender {
        self.tx.clone()
    }

    pub fn rt_handle(&self) -> tokio::runtime::Handle {
        self.rt.clone()
    }

    pub fn add_pending_promise(&mut self, promise: Handle) -> u64 {
        static NEXT_PROMISE_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_PROMISE_ID.fetch_add(1, Ordering::Relaxed);
        self.promises.insert(id, promise);
        id
    }

    pub fn take_promise(&mut self, id: u64) -> Handle {
        self.try_take_promise(id)
            .expect("Attempted to take a promise that was already taken")
    }

    pub fn try_take_promise(&mut self, id: u64) -> Option<Handle> {
        self.promises.remove(&id)
    }
}
