use std::any::Any;
use std::cell::RefCell;

use crate::gc::trace::Trace;
use crate::vm::local::LocalScope;
use crate::vm::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::Value;

#[derive(Debug)]
pub struct Array {
    items: RefCell<Vec<Value>>,
    obj: NamedObject,
}

impl Array {
    pub fn new(vm: &mut Vm) -> Self {
        Array {
            items: RefCell::new(Vec::new()),
            obj: NamedObject::new(vm),
        }
    }

    pub fn from_vec(vm: &mut Vm, values: Vec<Value>) -> Self {
        Array {
            items: RefCell::new(values),
            obj: NamedObject::new(vm),
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
        Ok(items.get(index).cloned().unwrap_or(Value::null()))
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
        Ok(())
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
}
