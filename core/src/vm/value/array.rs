use std::cell::RefCell;

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

impl Object for Array {
    fn get_property(&self, key: &str) -> Result<Value, Value> {
        let items = self.items.borrow();
        let index = key.parse::<usize>().unwrap();
        Ok(items.get(index).cloned().unwrap_or(Value::Null))
    }

    fn set_property(&self, key: &str, value: Value) -> Result<Value, Value> {
        Ok(Value::Undefined)
    }

    fn apply(&self, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }
}
