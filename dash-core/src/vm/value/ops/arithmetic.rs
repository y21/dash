use crate::vm::local::LocalScope;
use crate::vm::value::Typeof;
use crate::vm::value::Value;

use super::abstractions::conversions::ValueConversion;

impl Value {
    pub fn add(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let left = self.to_primitive(scope, None)?;
        let right = other.to_primitive(scope, None)?;

        let leftstr = matches!(left.type_of(), Typeof::String);
        let rightstr = matches!(right.type_of(), Typeof::String);

        if leftstr || rightstr {
            let lstr = left.to_string(scope)?;
            let rstr = right.to_string(scope)?;
            Ok(Value::String(format!("{lstr}{rstr}").into()))
        } else {
            let lnum = left.to_number(scope)?;
            let rnum = right.to_number(scope)?;
            Ok(Value::Number(lnum + rnum))
        }
    }

    pub fn sub(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::Number(lnum - rnum))
    }

    pub fn mul(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::Number(lnum * rnum))
    }

    pub fn div(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::Number(lnum / rnum))
    }

    pub fn rem(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::Number(lnum % rnum))
    }

    pub fn pow(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::Number(lnum.powf(rnum)))
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
            (Value::Object(a), Value::Object(b)) => Value::Boolean(a.as_ptr() == b.as_ptr()),
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

    pub fn bitor(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::Number((this | that) as f64))
    }

    pub fn bitxor(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::Number((this ^ that) as f64))
    }

    pub fn bitand(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::Number((this & that) as f64))
    }

    pub fn bitshl(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::Number((this << that) as f64))
    }

    pub fn bitshr(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::Number((this >> that) as f64))
    }

    pub fn bitushr(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        // TODO: >>>
        Ok(Value::Number((this >> that) as f64))
    }
}
