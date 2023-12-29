use std::ops::Deref;

use crate::Vm;

use super::Handle;

// TODO: document this
// TL;DR for now, `Persist<T>` adds reference counting to a `Handle<T>`,
// allowing it to be held safely for longer than any LocalScope
// NOTE: careful with cycles, this can leak
pub struct Persistent(Handle);

impl Persistent {
    pub fn new(vm: &mut Vm, handle: Handle) -> Self {
        let this = Self(handle.clone());
        // This function creates a strong reference, so increment
        this.inc_refcount();

        // "Inserting" twice is fine, since it is a HashSet
        vm.external_refs.insert(handle);

        this
    }
}

impl Persistent {
    // Used in tests
    #[allow(unused)]
    pub(crate) fn refcount(&self) -> u64 {
        self.0.refcount()
    }

    fn inc_refcount(&self) -> u64 {
        let refcount = self.0.refcount().checked_add(1).expect("Reference count overflowed");
        unsafe { self.0.set_refcount(refcount) };
        refcount
    }

    unsafe fn dec_refcount(&self) -> u64 {
        let refcount = self.0.refcount().checked_sub(1).expect("Reference count underflowed");
        self.0.set_refcount(refcount);
        refcount
    }
}

impl Deref for Persistent {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for Persistent {
    fn clone(&self) -> Self {
        self.inc_refcount();
        Self(self.0.clone())
    }
}

impl Drop for Persistent {
    fn drop(&mut self) {
        unsafe { self.dec_refcount() };

        // We don't have access to the external refs field of VM, so we can't remove it from there.
        // We instead do this during the *tracing* phase in the vm.

        // TODO: check if VM is detached, we need to deallocate manually here then
    }
}
