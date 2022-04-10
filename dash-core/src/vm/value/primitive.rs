use std::any::Any;
use std::iter;
use std::rc::Rc;

use crate::gc::trace::Trace;
use crate::throw;
use crate::vm::local::LocalScope;

use super::object::Object;
use super::object::PropertyKey;
use super::Typeof;
use super::Value;

pub const MAX_SAFE_INTEGER: u64 = 9007199254740991u64;
pub const MAX_SAFE_INTEGERF: f64 = 9007199254740991f64;

unsafe impl Trace for f64 {
    fn trace(&self) {}
}

impl Object for f64 {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        sc.statics.number_prototype.clone().get_property(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        // TODO: Reflect.setPrototypeOf(this, value); should throw
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.number_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        // TODO: error
        Ok(Value::undefined())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Number
    }
}

unsafe impl Trace for bool {
    fn trace(&self) {}
}

impl Object for bool {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        sc.statics.boolean_prototype.clone().get_property(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.boolean_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        // TODO: throw
        Ok(Value::undefined())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Boolean
    }
}

unsafe impl Trace for Rc<str> {
    fn trace(&self) {}
}

// TODO: impl<T: Deref<Target=O>, O: Object> Object for T  possible?
impl Object for Rc<str> {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        if let Some(value) = str::get_property(self, sc, key.clone())?.into_option() {
            return Ok(value);
        }

        sc.statics.string_prototype.clone().get_property(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.string_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, "string is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        str::own_keys(self)
    }

    fn type_of(&self) -> Typeof {
        str::type_of(self)
    }
}

pub fn array_like_keys(len: usize) -> impl Iterator<Item = Value> {
    (0..len)
        .map(|i| i.to_string())
        .chain(iter::once_with(|| "length".to_string()))
        .map(|x| Value::String(x.as_str().into()))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Undefined;
unsafe impl Trace for Undefined {
    fn trace(&self) {}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Null;
unsafe impl Trace for Null {
    fn trace(&self) {}
}

impl Object for Undefined {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        throw!(sc, "Cannot read property {:?} of undefined", key)
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        throw!(sc, "Cannot set property {:?} of undefined", key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        throw!(sc, "Cannot set prototype of undefined")
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, "Cannot get prototype of undefined")
    }

    fn apply<'s>(
        &self,
        sc: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(sc, "undefined is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Undefined
    }
}

impl Object for Null {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        throw!(sc, "Cannot read property {:?} of null", key)
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        throw!(sc, "Cannot set property {:?} of null", key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        throw!(sc, "Cannot set prototype of null")
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, "Cannot get prototype of null")
    }

    fn apply<'s>(
        &self,
        sc: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(sc, "null is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }
}

unsafe impl Trace for str {
    fn trace(&self) {}
}

impl Object for str {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        if let PropertyKey::String(st) = key {
            if st == "length" {
                return Ok(Value::Number(self.len() as f64));
            }

            if let Ok(index) = st.parse::<usize>() {
                let bytes = self.as_bytes();
                if let Some(&byte) = bytes.get(index) {
                    return Ok(Value::String((byte as char).to_string().into()));
                }
            }
        }

        Ok(Value::undefined())
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.string_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, "string is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        panic!("cannot convert string to any")
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        panic!("cannot convert string to any")
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(array_like_keys(self.len()).collect())
    }

    fn type_of(&self) -> Typeof {
        Typeof::String
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol(Rc<str>);

impl Symbol {
    pub fn new(description: Rc<str>) -> Self {
        Symbol(description)
    }
}

unsafe impl Trace for Symbol {
    fn trace(&self) {}
}

impl Object for Symbol {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        sc.statics.symbol_prototype.clone().get_property(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: Value,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.symbol_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        scope
            .statics
            .symbol_prototype
            .clone()
            .apply(scope, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Symbol
    }
}
