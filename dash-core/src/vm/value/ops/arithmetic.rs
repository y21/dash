use crate::vm::value::Value;

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
            _ => unimplemented!(),
        }
    }

    pub fn ne(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(*a != *b),
            _ => unimplemented!(),
        }
    }
}
