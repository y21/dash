use std::fmt::Write;

use dash_middle::interner::sym;
use dash_proc_macro::Trace;

use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::object::This;
use crate::{delegate, extract};

use super::function::args::CallArgs;
use super::object::{Object, OrdObject, PropertyValue};
use super::string::JsString;
use super::{Unrooted, Value};
use crate::value::propertykey::PropertyKey;

#[derive(Debug, Trace)]
pub struct Error {
    pub name: JsString,
    pub message: JsString,
    pub stack: JsString,
    pub obj: OrdObject,
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
    pub fn with_obj(obj: OrdObject, sc: &mut LocalScope<'_>, message: JsString) -> Self {
        let name = sym::Error.into();
        Self {
            name,
            message,
            stack: get_stack_trace(name, message, sc),
            obj,
        }
    }

    pub fn new(sc: &mut LocalScope<'_>, message: String) -> Self {
        let message = sc.intern(&*message).into();

        Self::with_obj(OrdObject::with_prototype(sc.statics.error_prototype), sc, message)
    }

    pub fn empty() -> Self {
        Self {
            name: sym::Error.into(),
            message: sym::empty.into(),
            stack: sym::empty.into(),
            obj: OrdObject::null(),
        }
    }

    pub fn empty_with_name(name: JsString) -> Self {
        Self {
            name,
            message: sym::empty.into(),
            stack: sym::empty.into(),
            obj: OrdObject::null(),
        }
    }
}

impl Object for Error {
    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        match key.to_js_string(sc) {
            Some(sym::name) => Ok(Some(PropertyValue::static_default(Value::string(self.name)))),
            Some(sym::message) => Ok(Some(PropertyValue::static_default(Value::string(self.message)))),
            Some(sym::stack) => Ok(Some(PropertyValue::static_default(Value::string(self.stack)))),
            _ => self.obj.get_property_descriptor(key, sc),
        }
    }

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope) -> Result<(), Value> {
        // TODO: this should special case name/stack
        self.obj.set_property(key, value, sc)
    }

    fn delete_property(&self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Value> {
        // TODO: delete/clear property
        self.obj.delete_property(key, sc)
    }

    fn apply(
        &self,
        callee: ObjectId,
        this: This,
        args: CallArgs,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(callee, this, args, scope)
    }

    fn set_prototype(&self, value: Value, sc: &mut LocalScope) -> Result<(), Value> {
        self.obj.set_prototype(value, sc)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.obj.own_keys(sc)
    }

    extract!(self);
}

// Other types of errors
macro_rules! define_error_type {
    ( $($s:ident, $t:expr, $proto:ident);* ) => {
        $(
            #[derive(Debug, Trace)]
            pub struct $s {
                pub inner: Error,
            }

            impl $s {
                pub fn new(vm: &mut LocalScope<'_>, message: String) -> Self {
                    let message = vm.intern(&*message).into();
                    let object = Self::object(vm);
                    Self::new_with_js_string(vm, object, message)
                }

                pub fn object(vm: &LocalScope<'_>) -> OrdObject {
                    OrdObject::with_prototype(vm.statics.$proto)
                }

                pub fn new_with_js_string(vm: &mut LocalScope<'_>, obj: OrdObject, message: JsString) -> Self {
                    let name = $t.into();

                    Self {
                        inner: Error {
                            name,
                            message,
                            stack: get_stack_trace(name, message, vm),
                            obj
                        }
                    }
                }

                pub fn empty() -> Self {
                    Self {
                        inner: Error::empty_with_name($t.into()),
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
                    apply,
                    own_keys
                );

                extract!(self, inner);
            }
        )*
    };
}

define_error_type!(
    EvalError, sym::EvalError, eval_error_prototype;
    RangeError, sym::RangeError, range_error_prototype;
    ReferenceError, sym::ReferenceError, reference_error_prototype;
    SyntaxError, sym::SyntaxError, syntax_error_prototype;
    TypeError, sym::TypeError, type_error_prototype;
    URIError, sym::URIError, uri_error_prototype;
    AggregateError, sym::AggregateError, aggregate_error_prototype
);
