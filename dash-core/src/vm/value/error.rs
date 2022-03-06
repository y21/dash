use std::rc::Rc;

use crate::gc::trace::Trace;
use crate::vm::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::Value;

#[derive(Debug)]
pub struct Error {
    pub name: Rc<str>,
    pub message: Rc<str>,
    pub obj: NamedObject,
}

impl Error {
    pub fn new<S: Into<Rc<str>>>(vm: &mut Vm, message: S) -> Self {
        Self {
            name: "Error".into(),
            message: message.into(),
            obj: NamedObject::new(vm),
        }
    }

    pub fn with_name<S1: Into<Rc<str>>, S2: Into<Rc<str>>>(
        vm: &mut Vm,
        name: S1,
        message: S2,
    ) -> Self {
        Self {
            name: name.into(),
            message: message.into(),
            obj: NamedObject::new(vm),
        }
    }
}

unsafe impl Trace for Error {
    fn trace(&self) {}
}

impl Object for Error {
    fn get_property(
        &self,
        sc: &mut crate::vm::local::LocalScope,
        key: &str,
    ) -> Result<super::Value, super::Value> {
        match key {
            "name" => Ok(Value::String(self.name.clone())),
            "message" => Ok(Value::String(self.message.clone())),
            _ => self.obj.get_property(sc, key),
        }
    }

    fn set_property(
        &self,
        sc: &mut crate::vm::local::LocalScope,
        key: &str,
        value: super::Value,
    ) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn apply<'s>(
        &self,
        scope: &mut crate::vm::local::LocalScope,
        this: super::Value,
        args: Vec<super::Value>,
    ) -> Result<super::Value, super::Value> {
        self.obj.apply(scope, this, args)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn set_prototype(
        &self,
        sc: &mut crate::vm::local::LocalScope,
        value: super::Value,
    ) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut crate::vm::local::LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }
}
