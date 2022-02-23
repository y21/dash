use crate::vm::value::Value;

impl Value {
    pub fn add(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(*a + *b),
            _ => unimplemented!(),
        }
    }
}
