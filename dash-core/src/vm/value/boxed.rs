use crate::gc::trace::Trace;
use crate::vm::local::LocalScope;
use crate::vm::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::Value;

#[derive(Debug)]
pub struct Number(f64, NamedObject);

impl Number {
    pub fn new(vm: &mut Vm, value: f64) -> Self {
        Self(value, NamedObject::new(vm))
    }

    pub fn with_obj(value: f64, obj: NamedObject) -> Self {
        Self(value, obj)
    }
}

unsafe impl Trace for Number {
    fn trace(&self) {}
}

impl Object for Number {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        self.1.get_property(sc, key)
    }

    fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        self.1.apply(sc, this, args)
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<Value, Value> {
        self.1.set_property(sc, key, value)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.1.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.1.get_prototype(sc)
    }
}
