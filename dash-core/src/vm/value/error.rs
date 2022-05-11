use std::any::Any;
use std::rc::Rc;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;
use crate::vm::local::LocalScope;
use crate::vm::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::object::PropertyKey;
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

    pub fn with_name<S1: Into<Rc<str>>, S2: Into<Rc<str>>>(vm: &mut Vm, name: S1, message: S2) -> Self {
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
        key: PropertyKey,
    ) -> Result<super::Value, super::Value> {
        match key {
            PropertyKey::String(s) if s == "name" => Ok(Value::String(self.name.clone())),
            PropertyKey::String(s) if s == "message" => Ok(Value::String(self.message.clone())),
            _ => self.obj.get_property(sc, key),
        }
    }

    fn set_property(
        &self,
        sc: &mut crate::vm::local::LocalScope,
        key: PropertyKey<'static>,
        value: super::Value,
    ) -> Result<(), super::Value> {
        todo!()
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        // TODO: delete/clear property
        Ok(Value::undefined())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: super::Value,
        args: Vec<super::Value>,
    ) -> Result<super::Value, super::Value> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn set_prototype(&self, sc: &mut crate::vm::local::LocalScope, value: super::Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut crate::vm::local::LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.obj.own_keys()
    }
}
