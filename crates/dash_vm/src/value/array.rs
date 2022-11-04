use std::any::Any;
use std::cell::Cell;
use std::cell::RefCell;

use dash_proc_macro::Trace;

use crate::delegate;
use crate::gc::handle::Handle;
use crate::local::LocalScope;
use crate::throw;
use crate::Vm;

use super::object::delegate_get_property;
use super::object::NamedObject;
use super::object::Object;
use super::object::PropertyKey;
use super::object::PropertyValue;
use super::object::PropertyValueKind;
use super::ops::abstractions::conversions::ValueConversion;
use super::primitive::array_like_keys;
use super::Value;

pub const MAX_LENGTH: usize = 4294967295;

#[derive(Debug, Trace)]
pub struct Array {
    items: RefCell<Vec<PropertyValue>>,
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

    pub fn from_vec(vm: &mut Vm, values: Vec<PropertyValue>) -> Self {
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

    pub fn inner(&self) -> &RefCell<Vec<PropertyValue>> {
        &self.items
    }
}

impl Object for Array {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        delegate_get_property(self, sc, key)
    }

    fn get_property_descriptor(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Option<PropertyValue>, Value> {
        let items = self.items.borrow();

        if let PropertyKey::String(key) = &key {
            if key == "length" {
                return Ok(Some(PropertyValue::static_default(Value::number(items.len() as f64))));
            }

            if let Ok(index) = key.parse::<usize>() {
                if index < MAX_LENGTH {
                    if let Some(element) = items.get(index).cloned() {
                        return Ok(Some(element));
                    }
                }
            }
        }

        self.obj.get_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        if let PropertyKey::String(key) = &key {
            let mut items = self.items.borrow_mut();

            if key == "length" {
                // TODO: this shouldnt be undefined
                let value = value.kind().get_or_apply(sc, Value::undefined())?;
                let new_len = value.to_number(sc)? as usize;

                if new_len > MAX_LENGTH {
                    throw!(sc, "Invalid array length");
                }

                items.resize(new_len as usize, PropertyValue::static_default(Value::undefined()));
                return Ok(());
            }

            if let Ok(index) = key.parse::<usize>() {
                if index < MAX_LENGTH {
                    if index >= items.len() {
                        items.resize(index + 1, PropertyValue::static_default(Value::undefined()));
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
                    let old = std::mem::replace(item, PropertyValue::static_default(Value::null()));
                    return Ok(match old.into_kind() {
                        PropertyValueKind::Static(value) => value,
                        PropertyValueKind::Trap { .. } => Value::undefined(),
                    });
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

#[derive(Debug, Trace)]
pub struct ArrayIterator {
    index: Cell<usize>,
    length: usize,
    value: Value,
    obj: NamedObject,
}

impl Object for ArrayIterator {
    delegate!(
        obj,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        own_keys
    );

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

pub fn spec_array_get_property(scope: &mut LocalScope, target: &Value, index: usize) -> Result<Value, Value> {
    // specialize array path
    if let Value::Object(obj) | Value::External(obj) = target {
        if let Some(arr) = obj.as_any().downcast_ref::<Array>() {
            let inner = arr.inner().borrow();
            return match inner.get(index) {
                Some(value) => value.get_or_apply(scope, Value::undefined()),
                None => Ok(Value::undefined()),
            };
        }
    }

    target.get_property(scope, index.to_string().into())
}

pub fn spec_array_set_property(
    scope: &mut LocalScope,
    target: &Value,
    index: usize,
    value: PropertyValue,
) -> Result<(), Value> {
    // specialize array path
    if let Value::Object(obj) | Value::External(obj) = target {
        if let Some(arr) = obj.as_any().downcast_ref::<Array>() {
            let mut inner = arr.inner().borrow_mut();

            if index < MAX_LENGTH {
                if index >= inner.len() {
                    inner.resize(index + 1, PropertyValue::static_default(Value::undefined()));
                }

                inner[index] = value;
                return Ok(());
            }
        }
    }

    target.set_property(scope, index.to_string().into(), value)
}
