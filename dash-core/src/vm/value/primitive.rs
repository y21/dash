use std::any::Any;
use std::rc::Rc;

use crate::gc::trace::Trace;
use crate::vm::local::LocalScope;

use super::object::Object;
use super::Value;

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
        Ok(Value::Undefined)
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
        Ok(Value::Undefined)
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
        // TODO: throw
        Ok(Value::Undefined)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
