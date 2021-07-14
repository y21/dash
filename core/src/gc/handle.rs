use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

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
    pub fn get_mut_unchecked(&self) -> &mut InnerHandle<T> {
        unsafe { &mut *self.0.as_ptr() }
    }

    /// Returns a reference to the underlying [InnerHandle]
    ///
    /// This does **not** check if it's already borrowed
    pub fn get_unchecked(&self) -> &InnerHandle<T> {
        unsafe { &*self.0.as_ptr() }
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
pub struct Handle<T>(*mut InnerHandleGuard<T>);

impl<T> Handle<T> {
    /// Creates a new [Handle]
    ///
    /// This operation is unsafe because its [Deref] implementation dereferences it
    pub unsafe fn new(ptr: *mut InnerHandleGuard<T>) -> Self {
        Self(ptr)
    }

    /// Returns a raw pointer to the underlying [InnerHandle]
    pub fn as_ptr(&self) -> *mut InnerHandleGuard<T> {
        self.0
    }
}

impl<T> Deref for Handle<T> {
    type Target = InnerHandleGuard<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
