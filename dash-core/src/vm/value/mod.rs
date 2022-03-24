pub mod array;
pub mod boxed;
pub mod conversions;
pub mod error;
pub mod function;
pub mod object;
pub mod ops;
pub mod primitive;

#[macro_export]
macro_rules! throw {
    ($vm:expr) => {
        return Err({
            let err = $crate::vm::value::error::Error::new($vm, "Unnamed error");
            $vm.gc_mut().register(err).into()
        })
    };
    ($vm:expr, $msg:expr) => {
        return Err({
            let err = $crate::vm::value::error::Error::new($vm, $msg);
            $vm.gc_mut().register(err).into()
        })
    };
    ($vm:expr, $msg:expr, $($arg:expr),*) => {
        return Err({
            let err = $crate::vm::value::error::Error::new($vm, format!($msg, $($arg),*));
            $vm.gc_mut().register(err).into()
        })
    };
}

use std::rc::Rc;

use crate::{
    compiler::constant::Constant,
    gc::{handle::Handle, trace::Trace},
    vm::value::function::FunctionKind,
};

use self::{
    function::{user::UserFunction, Function},
    object::Object,
};

use super::{local::LocalScope, Vm};
#[derive(Debug, Clone)]
pub enum Value {
    /// The number type
    Number(f64),
    /// The boolean type
    Boolean(bool),
    /// The string type
    String(Rc<str>),
    /// The undefined type
    Undefined,
    /// The null type
    Null,
    /// The object type
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
                let mut externals = Vec::new();

                for idx in f.externals.iter() {
                    let val = vm
                        .get_local(*idx as usize)
                        .expect("Referenced local not found");

                    let obj = match val {
                        Value::Object(o) => o,
                        // primitive types need to be put on the heap and GCd
                        // TODO: we need to update the locals in this current frame too
                        Value::Number(n) => vm.gc.register(n),
                        Value::Boolean(b) => vm.gc.register(b),
                        Value::String(s) => vm.gc.register(s),
                        _ => panic!("Expected object"),
                    };

                    externals.push(obj);
                }

                let uf =
                    UserFunction::new(f.buffer, f.constants, externals.into(), f.locals, f.params);
                let function = Function::new(vm, f.name, FunctionKind::User(uf));
                vm.gc.register(function).into()
            }
            Constant::Identifier(_) => unreachable!(),
        }
    }

    pub fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
        match self {
            Value::Object(handle) => handle.set_property(sc, key, value),
            Self::Number(n) => n.set_property(sc, key, value),
            Self::Boolean(b) => b.set_property(sc, key, value),
            Self::String(s) => s.set_property(sc, key, value),
            _ => unimplemented!(),
        }
    }

    pub fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        match self {
            Self::Object(o) => o.get_property(sc, key),
            Self::Number(n) => n.get_property(sc, key),
            Self::Boolean(b) => b.get_property(sc, key),
            Self::String(s) => s.get_property(sc, key),
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
            Self::Number(n) => n.apply(sc, this, args),
            Self::Boolean(b) => b.apply(sc, this, args),
            Self::String(s) => s.apply(sc, this, args),
            _ => unimplemented!(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Number(n) => *n != 0.0 && !n.is_nan(),
            _ => unimplemented!(),
        }
    }
}

pub trait ValueContext {
    fn unwrap_or_undefined(self) -> Value;
    fn unwrap_or_null(self) -> Value;
    fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value>;
}

impl ValueContext for Option<Value> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Some(x) => x,
            None => Value::Undefined,
        }
    }

    fn unwrap_or_null(self) -> Value {
        match self {
            Some(x) => x,
            None => Value::Null,
        }
    }

    fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value> {
        match self {
            Some(x) => Ok(x),
            None => throw!(vm, message),
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

    fn unwrap_or_null(self) -> Value {
        match self {
            Some(x) => x.clone(),
            None => Value::Null,
        }
    }

    fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value> {
        match self {
            Some(x) => Ok(x.clone()),
            None => throw!(vm, message),
        }
    }
}

impl<E> ValueContext for Result<Value, E> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Ok(x) => x,
            Err(_) => Value::Undefined,
        }
    }

    fn unwrap_or_null(self) -> Value {
        match self {
            Ok(x) => x,
            Err(_) => Value::Null,
        }
    }

    fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value> {
        match self {
            Ok(x) => Ok(x),
            Err(_) => throw!(vm, message),
        }
    }
}
