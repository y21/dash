use crate::vm::value::Value;

use super::abstractions::conversions::ValueConversion;

impl Value {
    pub fn add(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(*a + *b),
            _ => unimplemented!(),
        }
    }

    pub fn sub(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(*a - *b),
            _ => unimplemented!(),
        }
    }

    pub fn mul(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(*a * *b),
            _ => unimplemented!(),
        }
    }

    pub fn div(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(*a / *b),
            _ => unimplemented!(),
        }
    }

    pub fn rem(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(*a % *b),
            _ => unimplemented!(),
        }
    }

    pub fn pow(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a.powf(*b)),
            _ => unimplemented!(),
        }
    }

    pub fn lt(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(*a < *b),
            _ => unimplemented!(),
        }
    }

    pub fn le(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(*a <= *b),
            _ => unimplemented!(),
        }
    }

    pub fn gt(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(*a > *b),
            _ => unimplemented!(),
        }
    }

    pub fn ge(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(*a >= *b),
            _ => unimplemented!(),
        }
    }

    pub fn eq(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(*a == *b),
            _ => unimplemented!(),
        }
    }

    pub fn strict_eq(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(*a == *b),
            (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(*a == *b),
            _ => unimplemented!(),
        }
    }

    pub fn ne(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(*a != *b),
            _ => unimplemented!(),
        }
    }

    pub fn strict_ne(&self, other: &Self) -> Value {
        Value::Boolean(self == other)
    }

    pub fn not(&self) -> Value {
        Value::Boolean(!self.is_truthy())
    }

    pub fn bitor(&self, other: &Self) -> Result<Value, Value> {
        let this = self.to_int32()?;
        let that = other.to_int32()?;
        Ok(Value::Number((this | that) as f64))
    }

    pub fn bitxor(&self, other: &Self) -> Result<Value, Value> {
        let this = self.to_int32()?;
        let that = other.to_int32()?;
        Ok(Value::Number((this ^ that) as f64))
    }

    pub fn bitand(&self, other: &Self) -> Result<Value, Value> {
        let this = self.to_int32()?;
        let that = other.to_int32()?;
        Ok(Value::Number((this & that) as f64))
    }

    pub fn bitshl(&self, other: &Self) -> Result<Value, Value> {
        let this = self.to_int32()?;
        let that = other.to_int32()?;
        Ok(Value::Number((this << that) as f64))
    }

    pub fn bitshr(&self, other: &Self) -> Result<Value, Value> {
        let this = self.to_int32()?;
        let that = other.to_int32()?;
        Ok(Value::Number((this >> that) as f64))
    }

    pub fn bitushr(&self, other: &Self) -> Result<Value, Value> {
        let this = self.to_int32()?;
        let that = other.to_int32()?;
        // TODO: >>>
        Ok(Value::Number((this >> that) as f64))
    }
}
