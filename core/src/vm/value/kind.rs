use crate::vm::instruction::Constant;

use super::object::Object;

#[derive(Debug, Clone)]
pub enum ValueKind {
    Constant(Box<Constant>),
    Number(f64),
    Bool(bool),
    Object(Box<Object>),
    Undefined,
    Null,
}
