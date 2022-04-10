use std::{any::Any, borrow::Cow, cell::RefCell, collections::HashMap, fmt::Debug};

use crate::{
    gc::{handle::Handle, trace::Trace},
    throw,
    vm::{local::LocalScope, Vm},
};

use super::{ops::abstractions::conversions::ValueConversion, primitive::Symbol, Typeof, Value};

// only here for the time being, will be removed later
fn __assert_trait_object_safety(_: Box<dyn Object>) {}

pub trait Object: Debug + Trace {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value>;
    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value>;
    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value>;
    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value>;
    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value>;
    fn as_any(&self) -> &dyn Any;
    fn own_keys(&self) -> Result<Vec<Value>, Value>;
    fn type_of(&self) -> Typeof {
        Typeof::Object
    }
}

#[derive(Debug, Clone)]
pub struct NamedObject {
    prototype: RefCell<Option<Handle<dyn Object>>>,
    constructor: RefCell<Option<Handle<dyn Object>>>,
    values: RefCell<HashMap<PropertyKey<'static>, Value>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PropertyKey<'a> {
    String(Cow<'a, str>),
    Symbol(Symbol),
}

impl<'a> From<&'a str> for PropertyKey<'a> {
    fn from(s: &'a str) -> Self {
        PropertyKey::String(Cow::Borrowed(s))
    }
}

impl From<String> for PropertyKey<'static> {
    fn from(s: String) -> Self {
        PropertyKey::String(Cow::Owned(s))
    }
}

impl From<Symbol> for PropertyKey<'static> {
    fn from(s: Symbol) -> Self {
        PropertyKey::Symbol(s)
    }
}

impl<'a> PropertyKey<'a> {
    pub fn as_value(&self) -> Value {
        match self {
            PropertyKey::String(s) => Value::String(s.as_ref().into()),
            PropertyKey::Symbol(s) => Value::Symbol(s.clone()),
        }
    }

    pub fn from_value(sc: &mut LocalScope, value: Value) -> Result<Self, Value> {
        match value {
            Value::Symbol(s) => Ok(Self::Symbol(s.into())),
            other => {
                let key = other.to_string(sc)?;
                Ok(PropertyKey::String(key.to_string().into()))
            }
        }
    }
}

impl NamedObject {
    pub fn new(vm: &mut Vm) -> Self {
        let objp = vm.statics.object_prototype.clone();
        let objc = vm.statics.object_ctor.clone(); // TODO: function_ctor instead

        Self {
            prototype: RefCell::new(Some(objp)),
            constructor: RefCell::new(Some(objc)),
            values: RefCell::new(HashMap::new()),
        }
    }

    /// Creates an empty object with a null prototype
    pub fn null() -> Self {
        Self {
            prototype: RefCell::new(None),
            constructor: RefCell::new(None),
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_prototype_and_constructor(
        prototype: Handle<dyn Object>,
        ctor: Handle<dyn Object>,
    ) -> Self {
        Self {
            constructor: RefCell::new(Some(ctor)),
            prototype: RefCell::new(Some(prototype)),
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
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        if let PropertyKey::String(st) = &key {
            match st.as_ref() {
                "__proto__" => return self.get_prototype(sc),
                "constructor" => throw!(sc, "unimplemented"),
                _ => {}
            }
        };

        let values = self.values.borrow();
        if let Some(value) = values.get(&key) {
            return Ok(value.clone());
        }

        if let Some(prototype) = self.prototype.borrow().as_ref() {
            return prototype.get_property(sc, key);
        }

        Ok(Value::undefined())
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        let mut map = self.values.borrow_mut();
        map.insert(key, value);
        Ok(())
    }

    fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        match value {
            Value::Null(_) => self.prototype.replace(None),
            Value::Object(handle) => self.prototype.replace(Some(handle)),
            _ => throw!(sc, "prototype must be an object"),
        };

        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        let prototype = self.prototype.borrow();
        match prototype.as_ref() {
            Some(handle) => Ok(Value::Object(handle.clone())),
            None => Ok(Value::null()),
        }
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        let values = self.values.borrow();
        Ok(values.keys().map(PropertyKey::as_value).collect())
    }
}

impl Object for Handle<dyn Object> {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        (**self).get_property(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        (**self).set_property(sc, key, value)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        (**self).set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        (**self).get_prototype(sc)
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        (**self).apply(scope, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        (**self).own_keys()
    }

    fn type_of(&self) -> Typeof {
        (**self).type_of()
    }
}
