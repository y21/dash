use std::{any::Any, cell::RefCell, collections::HashMap, fmt::Debug};

use crate::{gc::trace::Trace, vm::local::LocalScope};

use super::Value;

// only here for the time being, will be removed later
fn __assert_trait_object_safety(_: Box<dyn Object>) {}

pub trait Object: Debug + Trace {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value>;
    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<Value, Value>;
    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct NamedObject {
    values: RefCell<HashMap<String, Value>>,
}

impl NamedObject {
    pub fn new() -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
        }
    }
}

unsafe impl Trace for NamedObject {
    fn trace(&self) {
        let values = self.values.borrow();
        for value in values.values() {
            value.trace();
        }
    }
}

impl Object for NamedObject {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        let map = self.values.borrow();
        map.get(key).cloned().ok_or(Value::Undefined)
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<Value, Value> {
        let mut map = self.values.borrow_mut();
        map.insert(key.into(), value);
        Ok(Value::Undefined)
    }

    fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        Ok(Value::Undefined)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
