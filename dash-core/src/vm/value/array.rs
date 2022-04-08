use std::any::Any;
use std::cell::RefCell;

use crate::gc::trace::Trace;
use crate::vm::local::LocalScope;
use crate::vm::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::primitive::array_like_keys;
use super::Value;

#[derive(Debug)]
pub struct Array {
    items: RefCell<Vec<Value>>,
    obj: NamedObject,
}

fn get_named_object(vm: &mut Vm) -> NamedObject {
    NamedObject::with_prototype_and_constructor(
        vm.statics.array_prototype.clone(),
        vm.statics.array_ctor.clone(),
    )
}

impl Array {
    pub fn new(vm: &mut Vm) -> Self {
        Array {
            items: RefCell::new(Vec::new()),
            obj: get_named_object(vm),
        }
    }

    pub fn from_vec(vm: &mut Vm, values: Vec<Value>) -> Self {
        Array {
            items: RefCell::new(values),
            obj: get_named_object(vm),
        }
    }

    pub fn with_obj(obj: NamedObject) -> Self {
        Self {
            items: RefCell::new(Vec::new()),
            obj,
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

        if key == "length" {
            return Ok(Value::Number(items.len() as f64));
        }

        if let Ok(index) = key.parse::<usize>() {
            if let Some(element) = items.get(index) {
                return Ok(element.clone());
            }
        }

        self.obj.get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
        if key == "length" {
            // swallow it
            // TODO: once we support defining non configurable properties, we can stop special casing this
            return Ok(());
        }

        if let Ok(index) = key.parse::<usize>() {
            let mut items = self.items.borrow_mut();
            if index >= items.len() {
                items.resize(index + 1, Value::undefined());
            }
            items[index] = value;
            return Ok(());
        }

        self.obj.set_property(sc, key, value)
    }

    fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        let items = self.items.borrow();
        Ok(array_like_keys(items.len()).collect())
    }
}
