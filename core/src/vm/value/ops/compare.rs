use crate::vm::value::{Value, ValueKind};

/// Represents the result of comparing two [Value]s
pub enum Compare {
    /// Lhs < Rhs
    Less,
    /// Lhs > Rhs
    Greater,
    /// Lhs == Rhs && Lhs <= Rhs && Lhs >= Rhs
    Equal,
}

impl Value {
    /// Compares two JavaScript values
    pub fn compare(&self, other: &Value) -> Option<Compare> {
        match &self.kind {
            ValueKind::Number(n) => {
                let rhs = other.as_number();
                if *n > rhs {
                    Some(Compare::Greater)
                } else if *n < rhs {
                    Some(Compare::Less)
                } else {
                    Some(Compare::Equal)
                }
            }
            ValueKind::Bool(b) => {
                let rhs = other.as_number();
                let lhs = *b as u8 as f64;

                if lhs > rhs {
                    Some(Compare::Greater)
                } else if lhs < rhs {
                    Some(Compare::Less)
                } else {
                    Some(Compare::Equal)
                }
            }
            _ => None,
        }
    }
}
