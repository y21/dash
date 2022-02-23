use crate::vm::value::Value;

pub trait ValueConversion {
    fn to_primitive(&self) -> Result<Value, Value>;
    fn to_number(&self) -> Result<f64, Value>;
    fn to_boolean(&self) -> Result<bool, Value>;
}

impl ValueConversion for Value {
    fn to_primitive(&self) -> Result<Value, Value> {
        todo!()
    }

    fn to_number(&self) -> Result<f64, Value> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => todo!(), // TODO: throw error
        }
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        todo!()
    }
}
