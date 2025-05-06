use std::cell::RefCell;

use ahash::HashMap;
use dash_proc_macro::Trace;

use crate::{Vm, delegate, extract};

use super::Value;
use super::object::{Object, OrdObject};

#[derive(Debug, Trace)]
pub struct Map {
    inner: RefCell<HashMap<Value, Value>>,
    obj: OrdObject,
}

impl Map {
    pub fn new(vm: &Vm) -> Self {
        Self::with_obj(OrdObject::with_prototype(vm.statics.map_prototype))
    }

    pub fn with_obj(obj: OrdObject) -> Self {
        Self {
            inner: RefCell::new(HashMap::default()),
            obj,
        }
    }

    pub fn set(&self, key: Value, value: Value) {
        self.inner.borrow_mut().insert(key, value);
    }

    pub fn has(&self, item: &Value) -> bool {
        self.inner.borrow().contains_key(item)
    }

    pub fn get(&self, item: &Value) -> Option<Value> {
        self.inner.borrow().get(item).cloned()
    }

    pub fn delete(&self, item: &Value) -> bool {
        self.inner.borrow_mut().remove(item).is_some()
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    pub fn size(&self) -> usize {
        self.inner.borrow().len()
    }
}

impl Object for Map {
    delegate!(
        obj,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        apply,
        own_keys
    );

    extract!(self);
}
