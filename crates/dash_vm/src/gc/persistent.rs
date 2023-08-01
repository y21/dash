use std::ops::Deref;

use super::Handle;

// TODO: document this
// TL;DR for now, `Persist<T>` adds reference counting to a `Handle<T>`,
// allowing it to be held safely for longer than any LocalScope
// NOTE: careful with cycles, this can leak
pub struct Persistent<T: ?Sized>(Handle<T>);

impl<T: ?Sized> Persistent<T> {
    pub fn new(handle: Handle<T>) -> Self {
        let this = Self(handle);
        // This function creates a strong reference, so increment
        this.inc_refcount();
        this
    }

    pub(crate) fn handle(&self) -> &Handle<T> {
        &self.0
    }

    fn inc_refcount(&self) -> u64 {
        let inner = unsafe { &*self.0.as_ptr() };
        let refcount = inner.refcount.get().checked_add(1).expect("Reference count overflowed");
        inner.refcount.set(refcount);
        refcount
    }

    unsafe fn dec_refcount(&self) -> u64 {
        let inner = &*self.0.as_ptr();
        let refcount = inner.refcount.get().checked_sub(1).expect("Reference count overflowed");
        inner.refcount.set(refcount);
        refcount
    }
}

impl<T: ?Sized> Deref for Persistent<T> {
    type Target = <Handle<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: ?Sized> Clone for Persistent<T> {
    fn clone(&self) -> Self {
        self.inc_refcount();
        Self::new(self.0.clone())
    }
}

impl<T: ?Sized> Drop for Persistent<T> {
    fn drop(&mut self) {
        unsafe { self.dec_refcount() };
        // TODO: check if VM is detached, we need to deallocate manually here then
    }
}
