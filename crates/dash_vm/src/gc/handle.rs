use std::cell::Cell;
use std::hash::Hash;
use std::ops::Deref;
use std::ptr::NonNull;

use bitflags::bitflags;

use crate::value::object::Object;

use super::trace::{Trace, TraceCtxt};

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

#[derive(Debug)]
pub struct Handle<T: ?Sized>(NonNull<GcNode<T>>);

impl<T: ?Sized> Handle<T> {
    /// # Safety
    /// The given [`NonNull`] pointer must point to a valid [`InnerHandle`]
    pub unsafe fn from_raw(ptr: NonNull<GcNode<T>>) -> Self {
        Self(ptr)
    }

    pub fn into_raw(ptr: Self) -> NonNull<GcNode<T>> {
        ptr.0
    }
}
impl<T: Object + 'static> Handle<T> {
    pub fn into_dyn(self) -> Handle<dyn Object> {
        let ptr = Handle::into_raw(self);
        // SAFETY: `T: Object` bound makes this cast safe. once CoerceUnsized is stable, we can use that instead.
        unsafe { Handle::<dyn Object>::from_raw(ptr) }
    }
}

impl<T: ?Sized> Eq for Handle<T> {}

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
}

impl<T: ?Sized> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}

unsafe impl<T: ?Sized + Trace> Trace for Handle<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        unsafe {
            let this = self.0.as_ref();
            if this.flags.is_marked() {
                // If already marked, do nothing to avoid getting stucked in an infinite loop
                return;
            }
            this.flags.mark();
        };

        T::trace(self, cx);
    }
}
