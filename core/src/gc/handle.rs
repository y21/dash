use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
};

use crate::vm::VM;

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
}
