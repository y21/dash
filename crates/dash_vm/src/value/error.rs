use std::any::Any;
use std::fmt::Write;
use std::rc::Rc;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;
use crate::local::LocalScope;
use crate::Vm;

use super::object::delegate_get_property;
use super::object::NamedObject;
use super::object::Object;
use super::object::PropertyKey;
use super::object::PropertyValue;
use super::Value;

#[derive(Debug)]
pub struct Error {
    pub name: Rc<str>,
    pub message: Rc<str>,
    pub stack: Rc<str>,
    pub obj: NamedObject,
}

fn get_stack_trace(name: &str, message: &str, vm: &Vm) -> Rc<str> {
    let mut stack = format!("{name}: {message}");

    for frame in vm.frames.iter().rev().take(10) {
        let name = frame.function.name.as_deref().unwrap_or("<anonymous>");
        let _ = write!(stack, "\n  at {name}");
    }

    stack.into()
}

impl Error {
    pub fn new<S: Into<Rc<str>>>(vm: &mut Vm, message: S) -> Self {
        Self::with_name(vm, "Error", message)
    }

    pub fn with_name<S1: Into<Rc<str>>, S2: Into<Rc<str>>>(vm: &mut Vm, name: S1, message: S2) -> Self {
        let name = name.into();
        let message = message.into();
        let stack = get_stack_trace(&name, &message, vm);

        Self {
            name,
            message,
            stack,
            obj: NamedObject::with_prototype_and_constructor(
                vm.statics.error_prototype.clone(),
                vm.statics.error_ctor.clone(),
            ),
        }
    }

    pub fn empty() -> Self {
        Self {
            name: "Error".into(),
            message: "".into(),
            stack: "".into(),
            obj: NamedObject::null(),
        }
    }
}

unsafe impl Trace for Error {
    fn trace(&self) {}
}

impl Object for Error {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<super::Value, super::Value> {
        delegate_get_property(self, sc, key)
    }

    fn get_property_descriptor(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Option<PropertyValue>, Value> {
        match key {
            PropertyKey::String(s) if s == "name" => {
                Ok(Some(PropertyValue::static_default(Value::String(self.name.clone()))))
            }
            PropertyKey::String(s) if s == "message" => {
                Ok(Some(PropertyValue::static_default(Value::String(self.message.clone()))))
            }
            PropertyKey::String(s) if s == "stack" => {
                Ok(Some(PropertyValue::static_default(Value::String(self.stack.clone()))))
            }
            _ => self.obj.get_property_descriptor(sc, key),
        }
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        // TODO: this should special case name/stack
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        // TODO: delete/clear property
        self.obj.delete_property(sc, key)
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

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.obj.own_keys()
    }
}
