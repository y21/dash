use std::any::Any;

use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::value::object::NamedObject;
use crate::value::object::Object;
use crate::value::Typeof;
use crate::value::Value;
use crate::Vm;

#[derive(Debug, Trace)]
pub struct BoundFunction {
    callee: Handle<dyn Object>,
    this: Option<Value>,
    args: Option<Vec<Value>>,
    obj: NamedObject,
}

impl BoundFunction {
    pub fn new(vm: &mut Vm, callee: Handle<dyn Object>, this: Option<Value>, args: Option<Vec<Value>>) -> Self {
        Self {
            callee,
            this,
            args,
            obj: NamedObject::new(vm),
        }
    }
}

impl Object for BoundFunction {
    fn get_property(
        &self,
        sc: &mut crate::local::LocalScope,
        key: crate::value::object::PropertyKey,
    ) -> Result<Value, Value> {
        self.obj.get_property(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut crate::local::LocalScope,
        key: crate::value::object::PropertyKey<'static>,
        value: crate::value::object::PropertyValue,
    ) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(
        &self,
        sc: &mut crate::local::LocalScope,
        key: crate::value::object::PropertyKey,
    ) -> Result<Value, Value> {
        self.obj.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut crate::local::LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut crate::local::LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        scope: &mut crate::local::LocalScope,
        _callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        let target_this = self.this.clone().unwrap_or(this);
        let target_args = self.args.clone().unwrap_or(args);

        self.callee.apply(scope, target_this, target_args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.obj.own_keys()
    }

    fn type_of(&self) -> Typeof {
        Typeof::Function
    }
}
