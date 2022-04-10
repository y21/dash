use std::{cell::Cell, fmt::Debug, hash::Hash, ops::Deref, ptr::NonNull};

use super::trace::Trace;

#[derive(Debug)]
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

impl<T: ?Sized> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

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

    pub fn replace(&mut self, new: Box<T>) {
        let t = unsafe { self.0.as_mut() };
        t.value = new;
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

// FIXME: this is severly unsound and hard to fix: this Handle can be smuggled because it's not tied to any reference
// and later, when its GC goes out of scope, it will be freed, even though it's still alive
impl<T: ?Sized> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.0.as_ref().value }
    }
}

impl<T: ?Sized> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}
