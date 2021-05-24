use super::value::{
    function::{FunctionKind, NativeFunction, UserFunction},
    object::Object,
    Value, ValueKind,
};
use std::cell::RefCell;
use std::rc::Rc;

impl From<NativeFunction> for Rc<RefCell<Value>> {
    fn from(f: NativeFunction) -> Self {
        Rc::new(RefCell::new(Value::new(ValueKind::Object(Box::new(
            Object::Function(FunctionKind::Native(f)),
        )))))
    }
}

impl From<Value> for Rc<RefCell<Value>> {
    fn from(v: Value) -> Self {
        Rc::new(RefCell::new(v))
    }
}

impl From<Object> for Value {
    fn from(o: Object) -> Self {
        Self::new(ValueKind::Object(Box::new(o)))
    }
}

impl From<UserFunction> for Value {
    fn from(f: UserFunction) -> Self {
        Self::new(ValueKind::Object(Box::new(Object::Function(
            FunctionKind::User(f),
        ))))
    }
}
