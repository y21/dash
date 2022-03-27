use std::any::Any;
use std::rc::Rc;

use crate::gc::trace::Trace;
use crate::throw;
use crate::vm::local::LocalScope;

use super::object::Object;
use super::Value;

pub const MAX_SAFE_INTEGER: u64 = 9007199254740991u64;
pub const MAX_SAFE_INTEGERF: f64 = 9007199254740991f64;

unsafe impl Trace for f64 {
    fn trace(&self) {}
}

impl Object for f64 {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        sc.statics.number_prototype.clone().get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
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
}

unsafe impl Trace for bool {
    fn trace(&self) {}
}

impl Object for bool {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        sc.statics.boolean_prototype.clone().get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
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
}

unsafe impl Trace for Rc<str> {
    fn trace(&self) {}
}

impl Object for Rc<str> {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        if let Some(value) = str::get_property(self, sc, key)?.into_option() {
            return Ok(value);
        }

        sc.statics.string_prototype.clone().get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
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
}

#[derive(Debug, Clone, Copy)]
pub struct Undefined;
unsafe impl Trace for Undefined {
    fn trace(&self) {}
}

#[derive(Debug, Clone, Copy)]
pub struct Null;
unsafe impl Trace for Null {
    fn trace(&self) {}
}

impl Object for Undefined {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        throw!(sc, "Cannot read property '{}' of undefined", key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
        throw!(sc, "Cannot set property '{}' of undefined", key)
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
}

impl Object for Null {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        throw!(sc, "Cannot read property '{}' of null", key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
        throw!(sc, "Cannot set property '{}' of null", key)
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
}

unsafe impl Trace for str {
    fn trace(&self) {}
}

impl Object for str {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        if key == "length" {
            return Ok(Value::Number(self.len() as f64));
        }

        if let Ok(index) = key.parse::<usize>() {
            let bytes = self.as_bytes();
            if let Some(&byte) = bytes.get(index) {
                return Ok(Value::String((byte as char).to_string().into()));
            }
        }

        Ok(Value::undefined())
    }

    fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
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
}
