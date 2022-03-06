use std::{cell::Cell, ops::Deref, ptr::NonNull};

use super::trace::Trace;

pub struct InnerHandle<T: ?Sized> {
    pub(crate) marked: Cell<bool>,
    pub(crate) value: Box<T>,
}

impl<T: ?Sized> InnerHandle<T> {
    pub fn mark(&self) {
        self.marked.set(true);
    }

    pub unsafe fn unmark(&self) {
        self.marked.set(false);
    }
}

#[derive(Debug)]
pub struct Handle<T: ?Sized>(NonNull<InnerHandle<T>>);

impl<T: ?Sized> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: ?Sized> Handle<T> {
    pub unsafe fn new(ptr: NonNull<InnerHandle<T>>) -> Self {
        Handle(ptr)
    }

    pub fn as_ptr(&self) -> *mut InnerHandle<T> {
        self.0.as_ptr()
    }
}

unsafe impl<T: ?Sized> Trace for Handle<T> {
    fn trace(&self) {
        unsafe {
            let this = self.0.as_ref();
            this.mark();
        };
    }
}

impl<T: ?Sized> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.0.as_ref().value }
    }
}
