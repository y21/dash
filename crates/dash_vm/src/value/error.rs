use std::any::Any;
use std::fmt::Write;
use std::rc::Rc;

use dash_proc_macro::Trace;

use crate::delegate;
use crate::gc::handle::Handle;
use crate::localscope::LocalScope;
use crate::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::object::PropertyKey;
use super::object::PropertyValue;
use super::Unrooted;
use super::Value;

#[derive(Debug, Trace)]
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
    pub fn new<S: Into<Rc<str>>>(vm: &Vm, message: S) -> Self {
        let ctor = vm.statics.error_ctor.clone();
        let proto = vm.statics.error_prototype.clone();
        Self::suberror(vm, "Error", message, ctor, proto)
    }

    pub fn suberror<S1: Into<Rc<str>>, S2: Into<Rc<str>>>(
        vm: &Vm,
        name: S1,
        message: S2,
        ctor: Handle<dyn Object>,
        proto: Handle<dyn Object>,
    ) -> Self {
        let name = name.into();
        let message = message.into();
        let stack = get_stack_trace(&name, &message, vm);

        Self {
            name,
            message,
            stack,
            obj: NamedObject::with_prototype_and_constructor(proto, ctor),
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

    pub fn empty_with_name<S: Into<Rc<str>>>(name: S) -> Self {
        Self {
            name: name.into(),
            message: "".into(),
            stack: "".into(),
            obj: NamedObject::null(),
        }
    }
}

impl Object for Error {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
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

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        // TODO: delete/clear property
        self.obj.delete_property(sc, key)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
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

// Other types of errors
macro_rules! define_error_type {
    ( $($t:ident $proto:ident $ctor:ident),* ) => {
        $(
            #[derive(Debug, Trace)]
            pub struct $t {
                pub inner: Error,
            }

            impl $t {
                pub fn new<S: Into<Rc<str>>>(vm: &Vm, message: S) -> Self {
                    let ctor = vm.statics.$ctor.clone();
                    let proto = vm.statics.$proto.clone();

                    Self {
                        inner: Error::suberror(vm, stringify!($t), message, ctor, proto),
                    }
                }

                pub fn empty() -> Self {
                    Self {
                        inner: Error::empty_with_name(stringify!($t)),
                    }
                }
            }

            impl Object for $t {
                delegate!(
                    inner,
                    get_own_property_descriptor,
                    get_property,
                    get_property_descriptor,
                    set_property,
                    delete_property,
                    set_prototype,
                    get_prototype,
                    as_any,
                    apply,
                    own_keys
                );
            }
        )*
    };
}

define_error_type!(
    EvalError eval_error_prototype eval_error_ctor,
    RangeError range_error_prototype range_error_ctor,
    ReferenceError reference_error_prototype reference_error_ctor,
    SyntaxError syntax_error_prototype syntax_error_ctor,
    TypeError type_error_prototype type_error_ctor,
    URIError uri_error_prototype uri_error_ctor,
    AggregateError aggregate_error_prototype aggregate_error_ctor
);
