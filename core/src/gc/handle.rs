use std::ops::{Deref, DerefMut};

/// A garbage collected handle to a T
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

    pub(crate) fn is_marked(&self) -> bool {
        self.marked
    }
}

impl<T> From<T> for InnerHandle<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

/// A handle that
pub struct Handle<T>(*mut InnerHandle<T>);

impl<T> Handle<T> {
    /// Creates a new [Handle]
    ///
    /// This operation is unsafe because its [Deref] implementation dereferences it
    pub unsafe fn new(ptr: *mut InnerHandle<T>) -> Self {
        Self(ptr)
    }

    /// Returns a raw pointer to the underlying [InnerHandle]
    pub fn as_ptr(&self) -> *mut InnerHandle<T> {
        self.0
    }
}

impl<T> Deref for Handle<T> {
    type Target = InnerHandle<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
