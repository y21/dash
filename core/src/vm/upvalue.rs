use std::{cell::RefCell, rc::Rc};

use super::value::Value;

/// A value from another frame
#[derive(Debug, Clone)]
pub struct Upvalue(pub Rc<RefCell<Value>>);
