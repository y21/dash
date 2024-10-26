use std::collections::hash_map::Entry;

use crate::{ExternalRefs, Vm};

use super::ObjectId;

// TODO: document this
// TL;DR for now, `Persist<T>` adds reference counting to a `Handle<T>`,
// allowing it to be held safely for longer than any LocalScope
// NOTE: careful with cycles, this can leak
pub struct Persistent(ObjectId, ExternalRefs);

impl Persistent {
    pub fn new(vm: &mut Vm, id: ObjectId) -> Self {
        let this = Self(id, vm.external_refs.clone());
        match vm.external_refs.0.borrow_mut().entry(id) {
            Entry::Occupied(mut entry) => {
                let value = entry.get_mut();
                *value = value.checked_add(1).unwrap();
            }
            Entry::Vacant(entry) => drop(entry.insert(1)),
        }

        this
    }
}

impl Persistent {
    // Used in tests
    #[allow(unused)]
    pub(crate) fn refcount(&self) -> u32 {
        (*self.1.0.borrow())[&self.0]
    }

    fn inc_refcount(&self) -> u32 {
        let mut map = self.1.0.borrow_mut();
        let val = map.get_mut(&self.0).unwrap();
        *val = val.checked_add(1).expect("reference count overflow");
        *val
    }

    unsafe fn dec_refcount(&self) -> u32 {
        let mut map = self.1.0.borrow_mut();
        let val = map.get_mut(&self.0).unwrap();
        *val = val.checked_sub(1).expect("reference count overflow");
        *val
    }

    pub fn id(&self) -> ObjectId {
        self.0
    }
}

impl Clone for Persistent {
    fn clone(&self) -> Self {
        self.inc_refcount();
        Self(self.0, self.1.clone())
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
