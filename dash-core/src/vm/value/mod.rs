pub mod array;
pub mod conversions;
pub mod error;
pub mod function;
pub mod object;
pub mod ops;

use std::rc::Rc;

use crate::{
    compiler::constant::Constant,
    gc::{handle::Handle, trace::Trace, Gc},
    vm::value::function::FunctionKind,
};

use self::{
    error::Error,
    function::{user::UserFunction, Function},
    object::Object,
};

use super::{local::LocalScope, Vm};
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(Rc<str>),
    Undefined,
    Null,
    Object(Handle<dyn Object>),
}

unsafe impl Trace for Value {
    fn trace(&self) {
        if let Value::Object(handle) = self {
            handle.trace();
        }
    }
}

impl Value {
    pub fn from_constant(constant: Constant, vm: &mut Vm) -> Self {
        match constant {
            Constant::Number(n) => Value::Number(n),
            Constant::Boolean(b) => Value::Boolean(b),
            Constant::String(s) => Value::String(s.into()),
            Constant::Undefined => Value::Undefined,
            Constant::Null => Value::Null,
            Constant::Function(f) => {
                let uf = UserFunction::new(f.buffer, f.constants, f.locals, f.params);
                let function = Function::new(f.name, FunctionKind::User(uf));
                vm.gc.register(function).into()
            }
            Constant::Identifier(_) => unreachable!(),
        }
    }

    pub fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        match self {
            Self::Object(o) => o.get_property(sc, key),
            _ => unimplemented!(),
        }
    }

    pub fn apply(
        &self,
        sc: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        match self {
            Value::Object(object) => object.apply(sc, this, args),
            _ => unimplemented!(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            _ => unimplemented!(),
        }
    }
}

pub trait ValueContext {
    fn unwrap_or_undefined(self) -> Value;
    fn context<S: Into<String>>(self, gc: &mut Gc<dyn Object>, message: S) -> Result<Value, Value>;
}

impl ValueContext for Option<Value> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Some(x) => x,
            None => Value::Undefined,
        }
    }

    fn context<S: Into<String>>(self, gc: &mut Gc<dyn Object>, message: S) -> Result<Value, Value> {
        match self {
            Some(x) => Ok(x),
            None => Err({
                let error = Error::new(message);
                gc.register(error).into()
            }),
        }
    }
}

impl ValueContext for Option<&Value> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Some(x) => x.clone(), // Values are cheap to clone
            None => Value::Undefined,
        }
    }

    fn context<S: Into<String>>(self, gc: &mut Gc<dyn Object>, message: S) -> Result<Value, Value> {
        match self {
            Some(x) => Ok(x.clone()),
            None => Err({
                let error = Error::new(message);
                gc.register(error).into()
            }),
        }
    }
}
