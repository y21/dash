use crate::jit::dataflow::join::{Join, JoinChange};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    Number,
    Boolean,
    /// The bottom value, no assumptions possible
    Any,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueMetadata {
    pub ty: ValueType,
}

impl Join for ValueMetadata {
    fn join(&self, other: &ValueMetadata) -> (ValueMetadata, JoinChange) {
        match (self.ty, other.ty) {
            (ValueType::Any, ValueType::Any)
            | (ValueType::Number, ValueType::Number)
            | (ValueType::Boolean, ValueType::Boolean) => (*self, JoinChange::Unchanged),
            _ => (ValueMetadata { ty: ValueType::Any }, JoinChange::Changed),
        }
    }
}
