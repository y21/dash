use super::{
    instruction::Constant,
    value::{
        array::Array,
        function::{Closure, FunctionKind, NativeFunction, UserFunction},
        object::{AnyObject, Object, Weak},
        weak::{WeakMap, WeakSet},
        Value, ValueKind,
    },
};
use std::{cell::RefCell, convert::TryFrom};
use std::{rc::Rc, str::Utf8Error};

impl From<Constant> for Value {
    fn from(c: Constant) -> Self {
        Self::new(ValueKind::Constant(Box::new(c)))
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::new(ValueKind::Number(n))
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::new(ValueKind::Bool(b))
    }
}

impl From<Object> for Value {
    fn from(o: Object) -> Self {
        Self::new(ValueKind::Object(Box::new(o)))
    }
}

impl From<&'static str> for Value {
    fn from(s: &'static str) -> Self {
        Object::String(s.to_owned()).into()
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Object::String(s).into()
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = Utf8Error;

    fn try_from(s: &[u8]) -> Result<Value, Self::Error> {
        std::str::from_utf8(s)
            .map(ToOwned::to_owned)
            .map(Object::String)
            .map(Into::into)
    }
}

impl From<FunctionKind> for Value {
    fn from(f: FunctionKind) -> Self {
        Object::Function(f).into()
    }
}

impl From<Array> for Value {
    fn from(a: Array) -> Self {
        Object::Array(a).into()
    }
}

impl From<AnyObject> for Value {
    fn from(o: AnyObject) -> Self {
        Object::Any(o).into()
    }
}

impl From<Closure> for Value {
    fn from(c: Closure) -> Self {
        FunctionKind::Closure(c).into()
    }
}

impl From<UserFunction> for Value {
    fn from(u: UserFunction) -> Self {
        FunctionKind::User(u).into()
    }
}

impl From<NativeFunction> for Value {
    fn from(f: NativeFunction) -> Self {
        FunctionKind::Native(f).into()
    }
}

impl From<Weak> for Value {
    fn from(s: Weak) -> Self {
        Object::Weak(s).into()
    }
}

impl From<WeakSet<RefCell<Value>>> for Value {
    fn from(s: WeakSet<RefCell<Value>>) -> Self {
        Weak::Set(s).into()
    }
}

impl From<WeakMap<RefCell<Value>, RefCell<Value>>> for Value {
    fn from(m: WeakMap<RefCell<Value>, RefCell<Value>>) -> Self {
        Weak::Map(m).into()
    }
}

impl From<Value> for Rc<RefCell<Value>> {
    fn from(v: Value) -> Self {
        Rc::new(RefCell::new(v))
    }
}
