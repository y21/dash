use crate::gc::Handle;

use super::object::Object;

/// The type of value
#[derive(Debug, Clone)]
pub enum ValueKind {
    /// A JavaScript number
    Number(f64),
    /// A JavaScript bool
    Bool(bool),
    /// A garbage collected object
    Object(Handle<Object>),
    /// JavaScript undefined value
    Undefined,
    /// JavaScript null value
    Null,
}
