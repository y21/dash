use std::cell::RefCell;
use std::ptr::NonNull;

use dash_vm::gc::handle::Handle;
use dash_vm::gc::handle::InnerHandle;
use dash_vm::value::object::Object;
use dash_vm::Vm;

use crate::event::EventSender;

pub struct State {
    rt: tokio::runtime::Handle,
    tx: EventSender,
    http_handler: RefCell<Option<NonNull<InnerHandle<dyn Object>>>>,
}

impl State {
    pub fn new(rt: tokio::runtime::Handle, tx: EventSender) -> Self {
        Self {
            rt,
            tx,
            http_handler: RefCell::new(None),
        }
    }

    pub(crate) fn set_http_handler(&self, handler: &Handle<dyn Object>) {
        let inner = NonNull::new(handler.as_ptr()).unwrap();
        self.http_handler.replace(Some(inner));
    }

    pub fn http_handler(&self) -> Option<Handle<dyn Object>> {
        self.http_handler.borrow().map(|ptr| unsafe { Handle::new(ptr) })
    }

    pub fn needs_event_loop(&self) -> bool {
        self.http_handler.borrow().is_some()
    }

    pub fn try_from_vm(vm: &Vm) -> Option<&Self> {
        vm.params().state()
    }

    pub fn event_sender(&self) -> EventSender {
        self.tx.clone()
    }

    pub fn rt_handle(&self) -> tokio::runtime::Handle {
        self.rt.clone()
    }
}
