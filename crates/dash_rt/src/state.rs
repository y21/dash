use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dash_vm::gc::persistent::Persistent;
use dash_vm::value::object::Object;
use dash_vm::Vm;

use crate::active_tasks::TaskIds;
use crate::event::EventSender;
use crate::module::ModuleLoader;

pub struct State {
    rt: tokio::runtime::Handle,
    tx: EventSender,
    root_module: Rc<RefCell<Option<Box<dyn ModuleLoader>>>>,
    tasks: TaskIds,
    promises: RefCell<HashMap<u64, Persistent<dyn Object>>>,
}

impl State {
    pub fn new(rt: tokio::runtime::Handle, tx: EventSender) -> Self {
        Self {
            rt,
            tx,
            root_module: Rc::new(RefCell::new(None)),
            tasks: TaskIds::new(),
            promises: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn set_root_module(&self, module: Box<dyn ModuleLoader>) {
        self.root_module.replace(Some(module));
    }

    pub fn root_module(&self) -> &Rc<RefCell<Option<Box<dyn ModuleLoader>>>> {
        &self.root_module
    }

    pub fn active_tasks(&self) -> &TaskIds {
        &self.tasks
    }

    pub fn needs_event_loop(&self) -> bool {
        self.tasks.has_tasks() || !self.promises.borrow().is_empty()
    }

    pub fn try_from_vm(vm: &Vm) -> Option<&Self> {
        vm.params().state()
    }

    /// Same as try_from_vm, but panics if state failed to downcast (i.e. Vm was not created by a runtime, or was changed at runtime)
    ///
    /// Usually it is a programmer error if downcasting fails, so this method is preferred
    pub fn from_vm(vm: &Vm) -> &Self {
        Self::try_from_vm(vm).unwrap()
    }

    pub fn event_sender(&self) -> EventSender {
        self.tx.clone()
    }

    pub fn rt_handle(&self) -> tokio::runtime::Handle {
        self.rt.clone()
    }

    pub fn add_pending_promise(&self, promise: Persistent<dyn Object>) -> u64 {
        static NEXT_PROMISE_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_PROMISE_ID.fetch_add(1, Ordering::Relaxed);
        self.promises.borrow_mut().insert(id, promise);
        id
    }

    pub fn take_promise(&self, id: u64) -> Persistent<dyn Object> {
        self.try_take_promise(id)
            .expect("Attempted to take a promise that was already taken")
    }

    pub fn try_take_promise(&self, id: u64) -> Option<Persistent<dyn Object>> {
        self.promises.borrow_mut().remove(&id)
    }
}
