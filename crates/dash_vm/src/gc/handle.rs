use std::any::Any;
use std::cell::Cell;
use std::fmt::Debug;
use std::hash::Hash;
use std::ptr::{addr_of, NonNull};

use bitflags::bitflags;

use crate::localscope::LocalScope;
use crate::value::object::{PropertyKey, PropertyValue};
use crate::value::primitive::PrimitiveCapabilities;
use crate::value::{Typeof, Unrooted, Value};

use super::trace::{Trace, TraceCtxt};

bitflags! {
    #[derive(Default)]
    pub struct HandleFlagsInner: u8 {
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

#[repr(C)]
#[allow(clippy::type_complexity)]
pub struct ObjectVTable {
    pub(crate) drop_boxed_gcnode: unsafe fn(*mut GcNode<()>),
    pub(crate) trace: unsafe fn(*const (), &mut TraceCtxt<'_>),
    pub(crate) debug_fmt: unsafe fn(*const (), &mut core::fmt::Formatter<'_>) -> core::fmt::Result,
    pub(crate) js_get_own_property:
        unsafe fn(*const (), &mut LocalScope<'_>, Value, PropertyKey) -> Result<Unrooted, Unrooted>,
    pub(crate) js_get_own_property_descriptor:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Option<PropertyValue>, Unrooted>,
    pub(crate) js_get_property: unsafe fn(*const (), &mut LocalScope, Value, PropertyKey) -> Result<Unrooted, Unrooted>,
    pub(crate) js_get_property_descriptor:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Option<PropertyValue>, Unrooted>,
    pub(crate) js_set_property:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey, PropertyValue) -> Result<(), Value>,
    pub(crate) js_delete_property: unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Unrooted, Value>,
    pub(crate) js_set_prototype: unsafe fn(*const (), &mut LocalScope<'_>, Value) -> Result<(), Value>,
    pub(crate) js_get_prototype: unsafe fn(*const (), &mut LocalScope<'_>) -> Result<Value, Value>,
    pub(crate) js_apply:
        unsafe fn(*const (), &mut LocalScope<'_>, Handle, Value, Vec<Value>) -> Result<Unrooted, Unrooted>,
    pub(crate) js_construct:
        unsafe fn(*const (), &mut LocalScope<'_>, Handle, Value, Vec<Value>) -> Result<Unrooted, Unrooted>,
    pub(crate) js_as_any: unsafe fn(*const ()) -> *const dyn Any,
    pub(crate) js_as_primitive_capable: unsafe fn(*const ()) -> Option<*const dyn PrimitiveCapabilities>,
    pub(crate) js_own_keys: unsafe fn(*const (), sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value>,
    pub(crate) js_type_of: unsafe fn(*const ()) -> Typeof,
}

#[repr(C, align(8))]
#[doc(hidden)]
pub struct Align8<T>(pub T);

#[repr(C)]
pub struct GcNode<T> {
    pub(crate) vtable: &'static ObjectVTable,
    pub(crate) flags: HandleFlags,
    /// Persistent<T> reference count
    pub(crate) refcount: Cell<u64>,
    /// The next pointer in the linked list of nodes
    pub(crate) next: Option<NonNull<GcNode<()>>>,

    /// The value. Typically seen in its type-erased form `()`.
    ///
    /// IMPORTANT: since we are using `GcNode<()>` for offsetting the `value` field
    /// but we've previously allocated the `GcNode` with a concrete type, we must make
    /// sure that the alignment is <= the max alignment of all other fields, since
    /// that could otherwise cause `value` in `GcNode<Concrete>` to have a different offset.
    /// `Align8` on its own does not guarantee a maximum alignment, instead this condition is checked in `register_gc!`.
    ///
    /// It must also be the last field, since offsetting the other fields would be wrong. This `T` is really `!Sized` in disguise
    pub(crate) value: Align8<T>,
}

#[repr(C)]
#[derive(Eq, Debug, PartialEq, Clone, Hash)]
pub struct Handle(NonNull<GcNode<()>>);

impl Handle {
    /// # Safety
    /// The given [`NonNull`] pointer must point to a valid [`GcNode`]
    pub unsafe fn from_raw(ptr: NonNull<GcNode<()>>) -> Self {
        Self(ptr)
    }
}

impl Handle {
    pub fn as_ptr<U>(&self) -> *mut GcNode<U> {
        self.0.as_ptr().cast()
    }

    pub fn as_erased_ptr(&self) -> *mut GcNode<()> {
        self.0.as_ptr()
    }

    pub fn erased_value(&self) -> *const () {
        unsafe { addr_of!((*self.0.as_ptr()).value.0) }
    }

    pub fn vtable(&self) -> &'static ObjectVTable {
        unsafe { (*self.0.as_ptr()).vtable }
    }

    /// Returns the `Persistent<T>` refcount.
    pub fn refcount(&self) -> u64 {
        unsafe { (*self.0.as_ptr()).refcount.get() }
    }

    /// # Safety
    /// The updated refcount must not be updated to a value such that a drop
    /// causes the refcount to drop zero while there are active `Handle`s
    pub unsafe fn set_refcount(&self, refcount: u64) {
        (*self.0.as_ptr()).refcount.set(refcount);
    }

    pub fn next(&self) -> Option<NonNull<GcNode<()>>> {
        unsafe { (*self.0.as_ptr()).next }
    }

    pub fn flags(&self) -> HandleFlagsInner {
        unsafe { (*self.0.as_ptr()).flags.flags.get() }
    }

    pub fn interior_flags(&self) -> &HandleFlags {
        unsafe { &(*self.0.as_ptr()).flags }
    }
}

unsafe impl Trace for Handle {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        unsafe {
            let flags = &(*self.0.as_ptr()).flags;
            if flags.is_marked() {
                // If already marked, do nothing to avoid getting stucked in an infinite loop
                return;
            }
            flags.mark();

            (self.vtable().trace)(self.erased_value(), cx)
        };
    }
}
