use std::{any::Any, collections::HashMap, fmt::Debug};

use super::Value;

// only here for the time being, will be removed later
fn __assert_trait_object_safety(_: Box<dyn Object>) {}

pub trait Object: Debug {
    fn get_property(&self, key: &str) -> Result<Value, Value>;
    fn set_property(&mut self, key: &str, value: Value) -> Result<Value, Value>;
    fn apply(&self, this: Value, args: Vec<Value>) -> Result<Value, Value>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct AnonymousObject {
    values: HashMap<String, Value>,
}

impl AnonymousObject {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl Object for AnonymousObject {
    fn get_property(&self, key: &str) -> Result<Value, Value> {
        Ok(self.values.get(key).cloned().unwrap_or(Value::Undefined))
    }

    fn set_property(&mut self, key: &str, value: Value) -> Result<Value, Value> {
        self.values.insert(key.into(), value);
        Ok(Value::Undefined)
    }

    fn apply(&self, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        Ok(Value::Undefined)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
