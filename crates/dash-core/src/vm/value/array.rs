use std::any::Any;
use std::cell::Cell;
use std::cell::RefCell;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;
use crate::throw;
use crate::vm::local::LocalScope;
use crate::vm::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::object::PropertyKey;
use super::ops::abstractions::conversions::ValueConversion;
use super::primitive::array_like_keys;
use super::Value;

pub const MAX_LENGTH: usize = 4294967295;

#[derive(Debug)]
pub struct Array {
    items: RefCell<Vec<Value>>,
    obj: NamedObject,
}

fn get_named_object(vm: &mut Vm) -> NamedObject {
    NamedObject::with_prototype_and_constructor(vm.statics.array_prototype.clone(), vm.statics.array_ctor.clone())
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
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        let items = self.items.borrow();

        if let PropertyKey::String(key) = &key {
            if key == "length" {
                return Ok(Value::Number(items.len() as f64));
            }

            if let Ok(index) = key.parse::<usize>() {
                if index < MAX_LENGTH {
                    if let Some(element) = items.get(index) {
                        return Ok(element.clone());
                    }
                }
            }
        }

        self.obj.get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        if let PropertyKey::String(key) = &key {
            let mut items = self.items.borrow_mut();

            if key == "length" {
                let len = items.len();
                let new_len = value.to_number(sc)? as usize;

                if new_len > MAX_LENGTH {
                    throw!(sc, "Invalid array length");
                }

                items.resize(new_len as usize, Value::undefined());
                return Ok(());
            }

            if let Ok(index) = key.parse::<usize>() {
                if index < MAX_LENGTH {
                    if index >= items.len() {
                        items.resize(index + 1, Value::undefined());
                    }

                    items[index] = value;
                    return Ok(());
                }
            }
        }

        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        if let PropertyKey::String(key) = &key {
            if key == "length" {
                return Ok(Value::undefined());
            }

            if let Ok(index) = key.parse::<usize>() {
                let mut items = self.items.borrow_mut();

                if let Some(item) = items.get_mut(index) {
                    let old = std::mem::replace(item, Value::null());
                    return Ok(old);
                }
            }
        }

        self.obj.delete_property(sc, key)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
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

#[derive(Debug)]
pub struct ArrayIterator {
    index: Cell<usize>,
    length: usize,
    value: Value,
    obj: NamedObject,
}

unsafe impl Trace for ArrayIterator {
    fn trace(&self) {
        self.value.trace();
    }
}

impl Object for ArrayIterator {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        self.obj.get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        self.obj.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }
}

impl ArrayIterator {
    pub fn new(sc: &mut LocalScope, value: Value) -> Result<Self, Value> {
        let length = value.length_of_array_like(sc)?;

        Ok(ArrayIterator {
            index: Cell::new(0),
            length,
            value,
            obj: NamedObject::with_prototype_and_constructor(
                sc.statics.array_iterator_prototype.clone(),
                sc.statics.object_ctor.clone(),
            ),
        })
    }

    pub fn empty() -> Self {
        Self {
            index: Cell::new(0),
            length: 0,
            value: Value::null(),
            obj: NamedObject::null(),
        }
    }

    pub fn next(&self, sc: &mut LocalScope) -> Result<Option<Value>, Value> {
        let index = self.index.get();

        if index < self.length {
            self.index.set(index + 1);
            self.value.get_property(sc, index.to_string().into()).map(Some)
        } else {
            Ok(None)
        }
    }
}
