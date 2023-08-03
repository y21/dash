use std::ops::Deref;

use crate::value::object::Object;
use crate::Vm;

use super::Handle;

// TODO: document this
// TL;DR for now, `Persist<T>` adds reference counting to a `Handle<T>`,
// allowing it to be held safely for longer than any LocalScope
// NOTE: careful with cycles, this can leak
pub struct Persistent<T: ?Sized>(Handle<T>);

impl Persistent<dyn Object> {
    pub fn new(vm: &mut Vm, handle: Handle<dyn Object>) -> Self {
        let this = Self(handle.clone());
        // This function creates a strong reference, so increment
        this.inc_refcount();

        // "Inserting" twice is fine, since it is a HashSet
        vm.external_refs.insert(handle);

        this
    }
}

impl<T: ?Sized> Persistent<T> {
    pub(crate) fn handle(&self) -> &Handle<T> {
        &self.0
    }

    // Used in tests
    pub(crate) fn refcount(&self) -> u64 {
        let inner = unsafe { &*self.0.as_ptr() };
        inner.refcount.get()
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
        Self(self.0.clone())
    }
}

impl<T: ?Sized> Drop for Persistent<T> {
    fn drop(&mut self) {
        unsafe { self.dec_refcount() };

        // We don't have access to the external refs field of VM, so we can't remove it from there.
        // We instead do this during the *tracing* phase in the vm.

        // TODO: check if VM is detached, we need to deallocate manually here then
    }
}
