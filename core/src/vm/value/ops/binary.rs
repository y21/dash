use crate::vm::value::{Value, ValueKind};

impl Value {
    /// Implements the addition operator
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

    /// Implements the subtraction operator
    pub fn sub(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this - other))
    }

    /// Implements the multiplication operator
    pub fn mul(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this * other))
    }

    /// Implements the division operator
    pub fn div(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this / other))
    }

    /// Implements the remainder operator
    pub fn rem(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this % other))
    }

    /// Implements the exponentation operator
    pub fn pow(&self, other: &Value) -> Value {
        let this = self.as_number();
        let other = other.as_number();
        Value::new(ValueKind::Number(this.powf(other)))
    }

    /// Implements the left shift operator
    pub fn left_shift(&self, other: &Value) -> Value {
        let this = self.as_32bit_number();
        let other = other.as_32bit_number();
        Value::new(ValueKind::Number((this << other) as f64))
    }

    /// Implements the right shift operator
    pub fn right_shift(&self, other: &Value) -> Value {
        let this = self.as_32bit_number();
        let other = other.as_32bit_number();
        Value::new(ValueKind::Number((this >> other) as f64))
    }

    /// Implements the unsigned right shift operator
    pub fn unsigned_right_shift(&self, _other: &Value) -> Value {
        todo!()
    }

    /// Implements the bitwise and operator
    pub fn bitwise_and(&self, other: &Value) -> Value {
        let this = self.as_32bit_number();
        let other = other.as_32bit_number();
        Value::new(ValueKind::Number((this & other) as f64))
    }

    /// Implements the bitwise or operator
    pub fn bitwise_or(&self, other: &Value) -> Value {
        let this = self.as_32bit_number();
        let other = other.as_32bit_number();
        Value::new(ValueKind::Number((this | other) as f64))
    }

    /// Implements the bitwise xor operator
    pub fn bitwise_xor(&self, other: &Value) -> Value {
        let this = self.as_32bit_number();
        let other = other.as_32bit_number();
        Value::new(ValueKind::Number((this ^ other) as f64))
    }

    /// Implements the bitwise not operator
    pub fn bitwise_not(&self) -> Value {
        let this = self.as_32bit_number();
        Value::new(ValueKind::Number(!this as f64))
    }
}
