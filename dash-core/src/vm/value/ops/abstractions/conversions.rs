use std::rc::Rc;

use crate::vm::local::LocalScope;
use crate::vm::value::Value;

pub trait ValueConversion {
    fn to_primitive(&self) -> Result<Value, Value>;
    fn to_number(&self) -> Result<f64, Value>;
    fn to_boolean(&self) -> Result<bool, Value>;
    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value>;
}

impl ValueConversion for Value {
    fn to_primitive(&self) -> Result<Value, Value> {
        todo!()
    }

    fn to_number(&self) -> Result<f64, Value> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => todo!(), // TODO: implement other cases
        }
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        match self {
            Value::Boolean(b) => Ok(*b),
            _ => todo!(), // TODO: implement other cases
        }
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Boolean(b) => Ok(b
                .then(|| sc.statics.get_true())
                .unwrap_or_else(|| sc.statics.get_false())),
            _ => todo!(), // TODO: implement other cases
        }
    }
}
