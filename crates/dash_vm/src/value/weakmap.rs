use dash_proc_macro::Trace;

use crate::{delegate, extract};

use super::Value;
use super::map::Map;
use super::object::{OrdObject, Object};

#[derive(Debug, Trace)]
pub struct WeakMap {
    // for now
    map: Map,
}

impl WeakMap {
    pub fn with_obj(object: OrdObject) -> Self {
        Self {
            map: Map::with_obj(object),
        }
    }

    pub fn null() -> Self {
        Self::with_obj(OrdObject::null())
    }

    pub fn set(&self, key: Value, value: Value) {
        self.map.set(key, value);
    }

    pub fn delete(&self, key: &Value) -> bool {
        self.map.delete(key)
    }

    pub fn get(&self, key: &Value) -> Option<Value> {
        self.map.get(key)
    }

    pub fn has(&self, key: &Value) -> bool {
        self.map.has(key)
    }
}

impl Object for WeakMap {
    delegate!(
        map,
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
