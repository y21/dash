use dash_vm::gc::handle::Handle;
use dash_vm::value::object::Object;

pub mod active_tasks;
pub mod event;
pub mod module;
pub mod runtime;
pub mod state;

#[derive(Clone)]
pub struct ThreadSafeHandle(Handle<dyn Object>);

impl ThreadSafeHandle {
    pub fn new(handle: Handle<dyn Object>) -> Self {
        Self(handle)
    }

    pub fn into_inner(self) -> Handle<dyn Object> {
        self.0
    }
}

unsafe impl Send for ThreadSafeHandle {}
unsafe impl Sync for ThreadSafeHandle {}
