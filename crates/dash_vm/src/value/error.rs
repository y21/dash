use std::any::Any;
use std::fmt::Write;

use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::gc::interner::sym;
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::{delegate, Vm};

use super::object::{NamedObject, Object, PropertyKey, PropertyValue};
use super::string::JsString;
use super::{Unrooted, Value};

#[derive(Debug, Trace)]
pub struct Error {
    pub name: JsString,
    pub message: JsString,
    pub stack: JsString,
    pub obj: NamedObject,
}

fn get_stack_trace(name: JsString, message: JsString, sc: &mut LocalScope<'_>) -> JsString {
    let name = name.res(sc);
    let message = message.res(sc);
    let mut stack = format!("{name}: {message}");

    for frame in sc.frames.iter().rev().take(10) {
        let name = frame
            .function
            .name
            .map(|s| sc.interner.resolve(s))
            .unwrap_or("<anonymous>");
        let _ = write!(stack, "\n  at {name}");
    }

    sc.intern(stack.as_ref()).into()
}

impl Error {
    pub fn new<S: Into<String>>(sc: &mut LocalScope<'_>, message: S) -> Self {
        let ctor = sc.statics.error_ctor.clone();
        let proto = sc.statics.error_prototype.clone();
        Self::suberror(sc, sym::Error, message, ctor, proto)
    }

    pub fn new_with_js_string<S: Into<JsString>>(sc: &mut LocalScope<'_>, message: S) -> Self {
        let ctor = sc.statics.error_ctor.clone();
        let proto = sc.statics.error_prototype.clone();
        Self::suberror_with_js_string(sc, sym::Error, message, ctor, proto)
    }

    pub fn suberror_with_js_string<S1: Into<JsString>, S2: Into<JsString>>(
        sc: &mut LocalScope<'_>,
        name: S1,
        message: S2,
        ctor: ObjectId,
        proto: ObjectId,
    ) -> Self {
        let name = name.into();
        let message = message.into();
        let stack = get_stack_trace(name, message, sc);

        Self {
            name,
            message,
            stack,
            obj: NamedObject::with_prototype_and_constructor(proto, ctor),
        }
    }

    pub fn suberror<S1: Into<JsString>, S2: Into<String>>(
        sc: &mut LocalScope<'_>,
        name: S1,
        message: S2,
        ctor: ObjectId,
        proto: ObjectId,
    ) -> Self {
        let name = name.into();
        let message = sc.intern(message.into().as_ref()).into();
        let stack = get_stack_trace(name, message, sc);

        Self {
            name,
            message,
            stack,
            obj: NamedObject::with_prototype_and_constructor(proto, ctor),
        }
    }

    pub fn empty() -> Self {
        Self {
            name: sym::Error.into(),
            message: sym::empty.into(),
            stack: sym::empty.into(),
            obj: NamedObject::null(),
        }
    }

    pub fn empty_with_name<S: Into<JsString>>(name: S) -> Self {
        Self {
            name: name.into(),
            message: sym::empty.into(),
            stack: sym::empty.into(),
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
            PropertyKey::String(s) if s.sym() == sym::name => {
                Ok(Some(PropertyValue::static_default(Value::string(self.name))))
            }
            PropertyKey::String(s) if s.sym() == sym::message => {
                Ok(Some(PropertyValue::static_default(Value::string(self.message))))
            }
            PropertyKey::String(s) if s.sym() == sym::stack => {
                Ok(Some(PropertyValue::static_default(Value::string(self.stack))))
            }
            _ => self.obj.get_property_descriptor(sc, key),
        }
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
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
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self, _: &Vm) -> &dyn Any {
        self
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.obj.own_keys(sc)
    }
}

// Other types of errors
macro_rules! define_error_type {
    ( $($s:ident, $t:expr, $proto:ident, $ctor:ident);* ) => {
        $(
            #[derive(Debug, Trace)]
            pub struct $s {
                pub inner: Error,
            }

            impl $s {
                pub fn new<S: Into<String>>(vm: &mut LocalScope<'_>, message: S) -> Self {
                    let ctor = vm.statics.$ctor.clone();
                    let proto = vm.statics.$proto.clone();

                    Self {
                        inner: Error::suberror(vm, $t, message, ctor, proto),
                    }
                }

                pub fn new_with_js_string<S: Into<JsString>>(vm: &mut LocalScope<'_>, message: S) -> Self {
                    let ctor = vm.statics.$ctor.clone();
                    let proto = vm.statics.$proto.clone();

                    Self {
                        inner: Error::suberror_with_js_string(vm, $t, message, ctor, proto),
                    }
                }

                pub fn empty() -> Self {
                    Self {
                        inner: Error::empty_with_name($t),
                    }
                }
            }

            impl Object for $s {
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
    EvalError, sym::EvalError, eval_error_prototype, eval_error_ctor;
    RangeError, sym::RangeError, range_error_prototype, range_error_ctor;
    ReferenceError, sym::ReferenceError, reference_error_prototype, reference_error_ctor;
    SyntaxError, sym::SyntaxError, syntax_error_prototype, syntax_error_ctor;
    TypeError, sym::TypeError, type_error_prototype, type_error_ctor;
    URIError, sym::URIError, uri_error_prototype, uri_error_ctor;
    AggregateError, sym::AggregateError, aggregate_error_prototype, aggregate_error_ctor
);
