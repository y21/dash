use std::cell::Cell;

use dash_proc_macro::Trace;

use crate::frame::This;
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::{Vm, delegate, extract};

use super::Unrooted;
use super::function::args::CallArgs;
use super::object::{OrdObject, Object};

#[derive(Debug, Trace)]
pub struct ArrayBuffer {
    storage: Vec<Cell<u8>>,
    obj: OrdObject,
}

impl ArrayBuffer {
    pub fn from_storage(vm: &Vm, storage: Vec<Cell<u8>>) -> Self {
        Self {
            storage,
            obj: OrdObject::with_prototype_and_constructor(
                vm.statics.arraybuffer_prototype,
                vm.statics.arraybuffer_ctor,
            ),
        }
    }

    pub fn new(vm: &Vm) -> Self {
        Self::with_capacity(
            0,
            OrdObject::with_prototype_and_constructor(vm.statics.arraybuffer_prototype, vm.statics.arraybuffer_ctor),
        )
    }

    pub fn with_capacity(capacity: usize, obj: OrdObject) -> Self {
        Self {
            storage: vec![Cell::new(0); capacity],
            obj,
        }
    }

    pub fn empty() -> Self {
        Self {
            storage: Vec::new(),
            obj: OrdObject::null(),
        }
    }

    pub fn storage(&self) -> &[Cell<u8>] {
        &self.storage
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.len() == 0
    }
}

impl Object for ArrayBuffer {
    delegate!(
        obj,
        get_own_property_descriptor, // TODO: byteLength
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        own_keys // TODO: byteLength
    );

    fn apply(
        &self,
        callee: ObjectId,
        this: This,
        args: CallArgs,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(callee, this, args, scope)
    }

    extract!(self);
}
