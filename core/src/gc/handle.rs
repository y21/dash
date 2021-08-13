use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crate::vm::{
    value::{Value, ValueKind},
    VM,
};

use super::GcVisitor;

/// An inner garbage collected value
///
/// If an [InnerHandle] does not get marked as visited by the next GC cycle,
/// it will get garbage collected
#[derive(Debug)]
pub struct InnerHandle<T> {
    value: T,
    marked: bool,
}

impl<T> InnerHandle<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            marked: false,
        }
    }

    pub(crate) fn mark_visited(&mut self) {
        self.marked = true;
    }

    pub(crate) fn unmark_visited(&mut self) {
        self.marked = false;
    }

    pub(crate) fn is_marked(&self) -> bool {
        self.marked
    }
}

impl<T> From<T> for InnerHandle<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T> Deref for InnerHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for InnerHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// A guarded handle to a garbage collected handle
///
/// It uses [RefCell] to ensure that no aliasing happens
#[derive(Debug)]
pub struct InnerHandleGuard<T>(RefCell<InnerHandle<T>>);

impl<T> InnerHandleGuard<T> {
    /// Returns a mutable reference to the underlying [InnerHandle]
    ///
    /// This does **not** check if it's already borrowed
    pub unsafe fn get_mut_unchecked(&self) -> &mut InnerHandle<T> {
        &mut *self.0.as_ptr()
    }

    /// Returns a reference to the underlying [InnerHandle]
    ///
    /// This does **not** check if it's already borrowed
    pub unsafe fn get_unchecked(&self) -> &InnerHandle<T> {
        &*self.0.as_ptr()
    }
}

impl<T> From<T> for InnerHandleGuard<T> {
    fn from(t: T) -> Self {
        Self(RefCell::new(t.into()))
    }
}

impl<T> Deref for InnerHandleGuard<T> {
    type Target = RefCell<InnerHandle<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A handle that
#[derive(Debug, Clone)]
pub struct Handle<T>(*mut InnerHandleGuard<T>, *const ());

impl<T> Handle<T> {
    /// Creates a new [Handle]
    ///
    /// # Safety
    /// This operation is unsafe because its [Deref] implementation dereferences it
    pub unsafe fn new(ptr: *mut InnerHandleGuard<T>, marker: *const ()) -> Self {
        Self(ptr, marker)
    }

    /// Returns a raw pointer to the underlying [InnerHandle]
    pub fn as_ptr(&self) -> *mut InnerHandleGuard<T> {
        self.0
    }

    /// Drops the inner data in place.
    ///
    /// # Safety
    /// It is very unlikely for a user to want to call this function,
    /// as it is almost impossible to do this operation safely.
    /// Explicitly dropping a handle that is registered by a GC by calling this function
    /// will lead to undefined behavior in the next GC cycle as it needs to be dereferenced
    /// to know whether the inner handle was visited.
    /// After a call to this function, it is also UB to attempt to do
    /// nearly everything with a handle, such as attempting to borrow the inner handle.
    pub unsafe fn drop_in_place(&self) {
        drop(Box::from_raw(self.0));
    }

    /// Returns a reference to the underlying [InnerHandleGuard]
    ///
    /// ## Panics
    /// This function causes a panic if this handle is not managed by the given VM
    pub fn get<'h>(&self, vm: &'h VM) -> &'h InnerHandleGuard<T> {
        assert!(self.check_marker(vm));
        unsafe { &*self.0 }
    }

    /// Returns a reference to the underlying [InnerHandleGuard] without checking if it's managed by the given VM
    ///
    /// ## Safety
    /// If the garbage collector has deallocated this handle, it is undefined behavior to call this function
    pub unsafe fn get_unchecked(&self) -> &InnerHandleGuard<T> {
        unsafe { &*self.0 }
    }

    /// Borrows the inner handle
    ///
    /// ## Panics
    /// This function causes a panic if this handle is not managed by the given VM or if RefCell::borrow() panics
    pub fn borrow<'v>(&self, vm: &'v VM) -> Ref<'v, InnerHandle<T>> {
        assert!(self.check_marker(vm));
        unsafe { (&*self.0).borrow() }
    }

    /// Borrows the inner handle mutably
    ///
    /// ## Panics
    /// This function causes a panic if this handle is not managed by the given VM or if RefCell::borrow() panics mutably
    pub fn borrow_mut<'v>(&self, vm: &'v VM) -> RefMut<'v, InnerHandle<T>> {
        assert!(self.check_marker(vm));
        unsafe { (&*self.0).borrow_mut() }
    }

    /// Borrows the inner handle without checking if it's managed by the given VM, and with no lifetime constraints
    ///
    /// ## Safety
    /// This function effectively allows the caller to outlive a VM because there are no lifetime constraints
    /// Doing so is undefined behavior
    pub unsafe fn borrow_unbounded(&self) -> Ref<InnerHandle<T>> {
        unsafe { (&*self.0).borrow() }
    }

    /// Borrows the inner handle mutably without checking if it's managed by the given VM, and with no lifetime constraints
    ///
    /// ## Safety
    /// This function effectively allows the caller to outlive a VM because there are no lifetime constraints
    /// Doing so is undefined behavior
    pub unsafe fn borrow_mut_unbounded(&self) -> RefMut<InnerHandle<T>> {
        unsafe { (&*self.0).borrow_mut() }
    }

    /// Checks whether this handle is managed by a given VM
    pub fn check_marker(&self, vm: &VM) -> bool {
        self.1 == vm.get_gc_marker()
    }

    /// Updates the inner marker that is used to ensure that calls to `borrow`
    /// only succeed for the same GC
    ///
    /// ## Safety
    /// This function is unsafe because it allows setting the inner marker
    pub unsafe fn set_marker(&mut self, marker: *const ()) {
        self.1 = marker;
    }
}

/// A handle that is independent of a GC
///
/// This may be useful for dealing with types that operate on handles without
/// having access to a GC
pub struct OwnedHandle<T: GcVisitor> {
    marker: Box<u8>,
    inner: *mut InnerHandleGuard<T>,
}

impl<T: GcVisitor> OwnedHandle<T> {
    /// Creates a new [OwnedHandle]
    pub fn new(value: T) -> Self {
        let marker = Box::new(0);
        let handle = InnerHandle::new(value);
        let handle = InnerHandleGuard(RefCell::new(handle));

        Self {
            inner: Box::into_raw(Box::new(handle)),
            marker,
        }
    }

    /// Registers a new value and returns a handle to it, with its marker
    /// set to the one of this OwnedHandle
    pub fn register(&self, value: T) -> Handle<T> {
        let ptr = Box::into_raw(Box::new(InnerHandleGuard::from(value)));
        unsafe { Handle::new(ptr, self.get_marker()) }
    }

    /// Returns a reference to the inner handle
    pub fn borrow(&self) -> Ref<'_, InnerHandle<T>> {
        unsafe { &*self.inner }.borrow()
    }

    /// Returns a reference to the inner handle mutably
    pub fn borrow_mut(&mut self) -> RefMut<'_, InnerHandle<T>> {
        unsafe { &*self.inner }.borrow_mut()
    }

    /// Returns this OwnedHandle as a regular Handle
    pub fn as_handle(&self) -> Handle<T> {
        unsafe { Handle::new(self.inner, self.get_marker()) }
    }

    pub(crate) fn get_marker(&self) -> *const () {
        *self.marker as *const ()
    }
}

impl<T: GcVisitor> Drop for OwnedHandle<T> {
    fn drop(&mut self) {
        // TODO: careful, dont double free.. somehow
        // TODO2: make sure we dont box-alias
        let handle = unsafe { Box::from_raw(self.inner) };
        let mut value = handle.borrow_mut();
        let mut cx = DropContext::new();
        unsafe { <T as GcVisitor>::drop_reachable(&mut value, &mut cx) };
    }
}

/// When an [OwnedHandle] goes out of scope, it needs to drop everything it can
/// before dropping itself. To avoid double free, it needs to keep track of
/// all the handles that were already freed.
/// Instead of calling [Handle::drop_in_place] directly, one should use the given DropContext
/// to drop handles
pub struct DropContext {
    dropped: HashSet<*mut ()>,
}

impl DropContext {
    /// Creates a new [DropContext]
    pub fn new() -> Self {
        Self {
            dropped: HashSet::new(),
        }
    }

    /// Drops a handle if it hasn't already been dropped
    pub unsafe fn drop<T>(&mut self, handle: &Handle<T>) {
        let ptr = handle.as_ptr();
        let not_dropped = self.dropped.insert(ptr.cast());
        if not_dropped {
            handle.drop_in_place();
        }
    }
}

#[test]
fn test_owned_handle() {
    let handle = {
        let js_value = Value::new(ValueKind::Number(123f64));
        OwnedHandle::new(js_value)
    };

    let prop = Value::new(ValueKind::Number(456f64));
    let handle2 = handle.register(prop);

    {
        let handle = handle.as_handle();
        unsafe {
            handle.borrow_mut_unbounded().set_property("test", handle2);
        }
    }
    drop(handle);
}
