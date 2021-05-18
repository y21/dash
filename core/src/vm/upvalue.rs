use std::{cell::RefCell, rc::Rc};

use super::value::Value;

#[derive(Debug, Clone)]
pub struct Upvalue(pub Rc<RefCell<Value>>);
