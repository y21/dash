use std::any::Any;
use std::cell::Cell;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;
use crate::local::LocalScope;
use crate::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::object::PropertyKey;
use super::object::PropertyValue;
use super::Value;

#[derive(Debug)]
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

unsafe impl Trace for ArrayBuffer {
    fn trace(&self) {
        self.obj.trace();
    }
}

impl Object for ArrayBuffer {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        // TODO: check if key == byteLength
        self.obj.get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        self.obj.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

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

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.obj.own_keys() // TODO: add byteLength
    }
}
