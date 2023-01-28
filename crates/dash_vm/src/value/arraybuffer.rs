use std::any::Any;
use std::cell::Cell;

use dash_proc_macro::Trace;

use crate::delegate;
use crate::gc::handle::Handle;
use crate::local::LocalScope;
use crate::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::Value;

#[derive(Debug, Trace)]
pub struct ArrayBuffer {
    storage: Vec<Cell<u8>>,
    obj: NamedObject,
}

impl ArrayBuffer {
    pub fn new(vm: &mut Vm) -> Self {
        Self::with_capacity(vm, 0)
    }

    pub fn with_capacity(vm: &mut Vm, capacity: usize) -> Self {
        let (proto, ctor) = (&vm.statics.arraybuffer_prototype, &vm.statics.arraybuffer_ctor);

        Self {
            storage: vec![Cell::new(0); capacity],
            obj: NamedObject::with_prototype_and_constructor(proto.clone(), ctor.clone()),
        }
    }

    pub fn empty() -> Self {
        Self {
            storage: Vec::new(),
            obj: NamedObject::null(),
        }
    }

    pub fn storage(&self) -> &[Cell<u8>] {
        &self.storage
    }

    pub fn len(&self) -> usize {
        self.storage.len()
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
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
