use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    hash::Hash,
    ops::Deref,
    ptr::NonNull,
};

use bitflags::bitflags;

use super::trace::Trace;

bitflags! {
    struct HandleFlagsInner: u8 {
        const MARKED_VISITED = 1 << 0;
        const VM_DETACHED = 1 << 1;
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct HandleFlags {
    flags: Cell<HandleFlagsInner>,
}

impl HandleFlags {
    pub fn new() -> Self {
        Self {
            flags: Cell::new(HandleFlagsInner::empty()),
        }
    }

    pub fn mark(&self) {
        self.flags.set(self.flags.get() | HandleFlagsInner::MARKED_VISITED);
    }

    pub unsafe fn unmark(&self) {
        self.flags.set(!(self.flags.get() & HandleFlagsInner::MARKED_VISITED));
    }

    pub fn is_marked(&self) -> bool {
        self.flags.get().contains(HandleFlagsInner::MARKED_VISITED)
    }

    pub fn detach_vm(&self) {
        self.flags.set(self.flags.get() | HandleFlagsInner::VM_DETACHED);
    }

    pub fn is_vm_detached(&self) -> bool {
        self.flags.get().contains(HandleFlagsInner::VM_DETACHED)
    }
}

#[derive(Debug)]
pub struct InnerHandle<T: ?Sized> {
    pub(crate) flags: HandleFlags,
    /// Persistent<T> reference count
    pub(crate) refcount: Cell<u64>,
    pub(crate) value: Box<T>,
}

impl<T: ?Sized> InnerHandle<T> {
    pub fn ref_count(&self) -> u64 {
        self.refcount.get()
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

unsafe impl<T: ?Sized + Trace> Trace for Handle<T> {
    fn trace(&self) {
        unsafe {
            let this = self.0.as_ref();
            this.flags.mark();
        };

        T::trace(self);
    }
}

unsafe impl<T: ?Sized + Trace> Trace for RefCell<T> {
    fn trace(&self) {
        T::trace(&RefCell::borrow(self));
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
