use std::cell::Cell;
use std::ops::Deref;
use std::ptr::NonNull;

use bitflags::bitflags;

use crate::value::object::Object;

bitflags! {
    #[derive(Default)]
    struct HandleFlagsInner: u8 {
        /// Whether the node has been visited in the last mark phase or not.
        const MARKED_VISITED = 1 << 0;
        const VM_DETACHED = 1 << 1;
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct HandleFlags {
    flags: Cell<HandleFlagsInner>,
}

impl HandleFlags {
    pub fn mark(&self) {
        self.flags.set(self.flags.get() | HandleFlagsInner::MARKED_VISITED);
    }

    /// # Safety
    /// Unmarking a [`Handle`] makes it available for deallocation in the next cycle.
    /// Calling this can introduce Undefined Behavior if a GC cycle triggers and this [`Handle`]
    /// is still live.
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

pub struct GcNode<T: ?Sized> {
    pub(crate) flags: HandleFlags,
    /// Persistent<T> reference count
    pub(crate) refcount: Cell<u64>,
    /// The next pointer in the linked list of nodes
    pub(crate) next: Option<NonNull<GcNode<dyn Object>>>,
    pub(crate) value: T,
}
pub struct Handle<T: ?Sized>(NonNull<GcNode<T>>);

impl<T: ?Sized> Handle<T> {
    /// # Safety
    /// The given [`NonNull`] pointer must point to a valid [`InnerHandle`]
    pub unsafe fn from_raw(ptr: NonNull<GcNode<T>>) -> Self {
        Self(ptr)
    }
}

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

// FIXME: this is severly unsound and hard to fix: this Handle can be smuggled because it's not tied to any reference
// and later, when its GC goes out of scope, it will be freed, even though it's still alive
impl<T: ?Sized> Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &(*self.as_ptr()).value }
    }
}

impl<T: ?Sized> Handle<T> {
    pub fn as_ptr(&self) -> *mut GcNode<T> {
        self.0.as_ptr()
    }

    #[allow(clippy::boxed_local)]
    pub fn replace(&mut self, new: Box<T>) {
        todo!()
        // let t = unsafe { self.0.as_mut() };
        // t.value = new;
    }
}
