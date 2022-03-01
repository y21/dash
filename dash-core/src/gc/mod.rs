use std::{cell::Cell, ptr::NonNull};

use crate::{gc::handle::InnerHandle, vm::value::object::Object};

use self::{
    handle::Handle,
    linkedlist::{LinkedList, Node},
    trace::Trace,
};

pub mod handle;
pub mod linkedlist;
pub mod trace;

pub struct Gc<T: ?Sized> {
    list: LinkedList<InnerHandle<T>>,
}

impl<T: ?Sized + Trace> Gc<T> {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    pub fn register<V>(&mut self, value: V) -> Handle<T>
    where
        V: IntoHandle<T, InnerHandle<T>>,
    {
        value.into_handle(&mut self.list)
    }
}

pub unsafe trait IntoHandle<T: ?Sized, U> {
    fn into_handle(self, link: &mut LinkedList<U>) -> Handle<T>;
}

unsafe impl<T: Object + 'static> IntoHandle<dyn Object, InnerHandle<dyn Object>> for T {
    fn into_handle(self, link: &mut LinkedList<InnerHandle<dyn Object>>) -> Handle<dyn Object> {
        let handle: InnerHandle<dyn Object> = InnerHandle {
            marked: Cell::new(false),
            value: Box::new(self),
        };

        let node = Node {
            next: None,
            value: handle,
        };

        unsafe {
            let ptr = NonNull::new_unchecked(link.add(Box::new(node)));
            Handle::new(ptr)
        }
    }
}
