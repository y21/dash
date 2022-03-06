pub mod array;
pub mod boxed;
pub mod conversions;
pub mod error;
pub mod function;
pub mod object;
pub mod ops;

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
                let function = Function::new(vm, f.name, FunctionKind::User(uf));
                vm.gc.register(function).into()
            }
            Constant::Identifier(_) => unreachable!(),
        }
    }

    pub fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        match self {
            Self::Object(o) => o.get_property(sc, key),
            Self::Number(_) => sc.statics.number_prototype.clone().get_property(sc, key),
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
