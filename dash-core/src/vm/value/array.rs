use std::cell::RefCell;

use crate::gc::trace::Trace;
use crate::vm::local::LocalScope;

use super::object::Object;
use super::Value;

#[derive(Debug)]
pub struct Array {
    items: RefCell<Vec<Value>>,
}

impl Array {
    pub fn new() -> Self {
        Array {
            items: RefCell::new(Vec::new()),
        }
    }
}

impl From<Vec<Value>> for Array {
    fn from(values: Vec<Value>) -> Self {
        Array {
            items: RefCell::new(values),
        }
    }
}

unsafe impl Trace for Array {
    fn trace(&self) {
        let items = self.items.borrow();
        for item in items.iter() {
            item.trace();
        }
    }
}

impl Object for Array {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        let items = self.items.borrow();
        let index = key.parse::<usize>().unwrap();
        Ok(items.get(index).cloned().unwrap_or(Value::Null))
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<Value, Value> {
        Ok(Value::Undefined)
    }

    fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }
}
