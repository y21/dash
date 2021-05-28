use crate::{js_std, util};

use super::function::Constructor;
use super::{object::PropertyLookup, Value, ValueKind};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Array {
    pub elements: Vec<Rc<RefCell<Value>>>,
}

impl Array {
    pub fn new(elements: Vec<Rc<RefCell<Value>>>) -> Self {
        Self { elements }
    }

    pub fn at(&self, idx: impl Into<usize>) -> Option<Rc<RefCell<Value>>> {
        self.elements.get(idx.into()).cloned()
    }

    pub fn get_property_unboxed(&self, k: &str) -> Option<PropertyLookup> {
        match k {
            "length" => Some(PropertyLookup::Value(ValueKind::Number(
                self.elements.len() as f64,
            ))),
            "push" => Some(PropertyLookup::Function(
                js_std::array::push,
                "push",
                Constructor::NoCtor,
            )),
            _ => {
                if util::is_numeric(k) {
                    let idx = k.parse::<usize>().unwrap();
                    self.at(idx).map(PropertyLookup::ValueRef)
                } else {
                    None
                }
            }
        }
    }
}
