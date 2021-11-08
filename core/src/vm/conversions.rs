use crate::gc::Handle;

use super::value::{
    array::Array,
    function::{Closure, FunctionKind, Module, NativeFunction, UserFunction},
    generator::GeneratorIterator,
    object::{ExoticObject, Object, ObjectKind},
    promise::Promise,
    symbol::Symbol,
    weak::{Weak, WeakMap, WeakSet},
    Value, ValueKind,
};
use std::str::Utf8Error;
use std::{cell::RefCell, convert::TryFrom};

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

impl From<Handle<Object>> for Value {
    fn from(o: Handle<Object>) -> Self {
        Self::new(ValueKind::Object(o))
    }
}

impl From<ObjectKind> for Object {
    fn from(o: ObjectKind) -> Self {
        Object::new(o)
    }
}

impl From<ExoticObject> for Object {
    fn from(o: ExoticObject) -> Self {
        ObjectKind::Exotic(o).into()
    }
}

impl From<&'static str> for Object {
    fn from(s: &'static str) -> Self {
        ExoticObject::String(s.to_owned()).into()
    }
}

impl From<String> for Object {
    fn from(s: String) -> Self {
        ExoticObject::String(s).into()
    }
}

impl TryFrom<&[u8]> for Object {
    type Error = Utf8Error;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        std::str::from_utf8(s)
            .map(ToOwned::to_owned)
            .map(ExoticObject::String)
            .map(Into::into)
    }
}

impl From<FunctionKind> for Object {
    fn from(f: FunctionKind) -> Self {
        ExoticObject::Function(f).into()
    }
}

impl From<Array> for Object {
    fn from(a: Array) -> Self {
        ExoticObject::Array(a).into()
    }
}

impl From<Closure> for Object {
    fn from(c: Closure) -> Self {
        FunctionKind::Closure(c).into()
    }
}

impl From<UserFunction> for Object {
    fn from(u: UserFunction) -> Self {
        FunctionKind::User(u).into()
    }
}

impl From<NativeFunction> for Object {
    fn from(f: NativeFunction) -> Self {
        FunctionKind::Native(f).into()
    }
}

impl From<Module> for Object {
    fn from(f: Module) -> Self {
        FunctionKind::Module(f).into()
    }
}

impl From<Weak> for Object {
    fn from(s: Weak) -> Self {
        ExoticObject::Weak(s).into()
    }
}

impl From<Symbol> for Object {
    fn from(s: Symbol) -> Self {
        ExoticObject::Symbol(s).into()
    }
}

impl From<GeneratorIterator> for Object {
    fn from(g: GeneratorIterator) -> Self {
        ExoticObject::GeneratorIterator(g).into()
    }
}

impl From<WeakSet<RefCell<Value>>> for Object {
    fn from(s: WeakSet<RefCell<Value>>) -> Self {
        Weak::Set(s).into()
    }
}

impl From<WeakMap<RefCell<Value>, RefCell<Value>>> for Object {
    fn from(m: WeakMap<RefCell<Value>, RefCell<Value>>) -> Self {
        Weak::Map(m).into()
    }
}

impl From<Promise> for Object {
    fn from(p: Promise) -> Self {
        ExoticObject::Promise(p).into()
    }
}

impl From<UserFunction> for FunctionKind {
    fn from(f: UserFunction) -> Self {
        Self::User(f)
    }
}
