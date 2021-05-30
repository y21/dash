use crate::vm::value::{Value, ValueKind};

impl Value {
    pub fn add(&self, other: &Value) -> Value {
        match &self.kind {
            ValueKind::Object(_) => {
                let left = String::from(self.to_string());
                let right = other.to_string();

                Value::from(left + &right).into()
            }
            _ => {
                let this = self.as_number();
                let other = other.as_number();
                Value::from(this + other).into()
            }
        }
    }

    pub fn sub(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this - other))
    }

    pub fn mul(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this * other))
    }

    pub fn div(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this / other))
    }

    pub fn rem(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this % other))
    }

    pub fn pow(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this.powf(other)))
    }

    pub fn left_shift(&self, other: &Value) -> Value {
        let this = self.as_whole_number();
        let other = other.as_whole_number();
        Value::new(ValueKind::Number((this << other) as f64))
    }

    pub fn right_shift(&self, other: &Value) -> Value {
        let this = self.as_whole_number();
        let other = other.as_whole_number();
        Value::new(ValueKind::Number((this >> other) as f64))
    }

    pub fn unsigned_right_shift(&self, _other: &Value) -> Value {
        todo!()
    }

    pub fn bitwise_and(&self, other: &Value) -> Value {
        let this = self.as_whole_number();
        let other = other.as_whole_number();
        Value::new(ValueKind::Number((this & other) as f64))
    }

    pub fn bitwise_or(&self, other: &Value) -> Value {
        let this = self.as_whole_number();
        let other = other.as_whole_number();
        Value::new(ValueKind::Number((this | other) as f64))
    }

    pub fn bitwise_xor(&self, other: &Value) -> Value {
        let this = self.as_whole_number();
        let other = other.as_whole_number();
        Value::new(ValueKind::Number((this ^ other) as f64))
    }
}
