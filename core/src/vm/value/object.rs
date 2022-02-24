use std::{any::Any, cell::RefCell, collections::HashMap, fmt::Debug};

use super::Value;

// only here for the time being, will be removed later
fn __assert_trait_object_safety(_: Box<dyn Object>) {}

pub trait Object: Debug {
    fn get_property(&self, key: &str) -> Result<Value, Value>;
    fn set_property(&self, key: &str, value: Value) -> Result<Value, Value>;
    fn apply(&self, this: Value, args: Vec<Value>) -> Result<Value, Value>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct AnonymousObject {
    values: RefCell<HashMap<String, Value>>,
}

impl AnonymousObject {
    pub fn new() -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
        }
    }
}

impl Object for AnonymousObject {
    fn get_property(&self, key: &str) -> Result<Value, Value> {
        let map = self.values.borrow();
        map.get(key).cloned().ok_or(Value::Undefined)
    }

    fn set_property(&self, key: &str, value: Value) -> Result<Value, Value> {
        let mut map = self.values.borrow_mut();
        map.insert(key.into(), value);
        Ok(Value::Undefined)
    }

    fn apply(&self, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        Ok(Value::Undefined)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
