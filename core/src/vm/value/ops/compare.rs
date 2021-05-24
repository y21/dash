use crate::vm::value::{Value, ValueKind};

pub enum Compare {
    Less,
    Greater,
    Equal,
}

impl Value {
    pub fn compare(&self, other: &Value) -> Option<Compare> {
        match &self.kind {
            ValueKind::Number(n) => {
                let rhs = other.as_number();
                if *n > rhs {
                    Some(Compare::Greater)
                } else {
                    Some(Compare::Less)
                }
            }
            ValueKind::Bool(b) => {
                let rhs = other.as_number();
                let lhs = *b as u8 as f64;

                if lhs > rhs {
                    Some(Compare::Greater)
                } else {
                    Some(Compare::Less)
                }
            }
            _ => None,
        }
    }
}
