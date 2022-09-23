use std::cell::RefCell;
use std::rc::Rc;

use dash_vm::Vm;

use crate::active_tasks::TaskIds;
use crate::event::EventSender;
use crate::module::ModuleLoader;

pub struct State {
    rt: tokio::runtime::Handle,
    tx: EventSender,
    root_module: Rc<RefCell<Option<Box<dyn ModuleLoader>>>>,
    tasks: TaskIds,
}

impl State {
    pub fn new(rt: tokio::runtime::Handle, tx: EventSender) -> Self {
        Self {
            rt,
            tx,
            root_module: Rc::new(RefCell::new(None)),
            tasks: TaskIds::new(),
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
}
