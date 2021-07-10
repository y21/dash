use crate::vm::instruction::Constant;

use super::object::Object;

/// The type of value
#[derive(Debug, Clone)]
pub enum ValueKind {
    /// A compiled constant
    Constant(Box<Constant>),
    /// A JavaScript number
    Number(f64),
    /// A JavaScript bool
    Bool(bool),
    /// An object that owns a heap allocation
    Object(Box<Object>),
    /// JavaScript undefined value
    Undefined,
    /// JavaScript null value
    Null,
}
