use std::any::Any;
use std::cell::RefCell;

use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;
use crate::local::LocalScope;
use crate::PromiseAction;
use crate::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::Typeof;
use super::Value;

#[derive(Debug)]
pub enum PromiseState {
    Pending {
        resolve: Vec<Handle<dyn Object>>,
        reject: Vec<Handle<dyn Object>>,
    },
    Resolved(Value),
    Rejected(Value),
}

#[derive(Debug)]
pub struct Promise {
    state: RefCell<PromiseState>,
    obj: NamedObject,
}

unsafe impl Trace for Promise {
    fn trace(&self) {
        self.obj.trace();
    }
}

impl Promise {
    pub fn new(vm: &mut Vm) -> Self {
        Self {
            state: RefCell::new(PromiseState::Pending {
                reject: Vec::new(),
                resolve: Vec::new(),
            }),
            obj: NamedObject::with_prototype_and_constructor(
                vm.statics.promise_proto.clone(),
                vm.statics.promise_ctor.clone(),
            ),
        }
    }
    pub fn resolved(vm: &mut Vm, value: Value) -> Self {
        Self {
            state: RefCell::new(PromiseState::Resolved(value)),
            obj: NamedObject::with_prototype_and_constructor(
                vm.statics.promise_proto.clone(),
                vm.statics.promise_ctor.clone(),
            ),
        }
    }
    pub fn rejected(vm: &mut Vm, value: Value) -> Self {
        Self {
            state: RefCell::new(PromiseState::Rejected(value)),
            obj: NamedObject::with_prototype_and_constructor(
                vm.statics.promise_proto.clone(),
                vm.statics.promise_ctor.clone(),
            ),
        }
    }
    pub fn state(&self) -> &RefCell<PromiseState> {
        &self.state
    }
}

impl Object for Promise {
    fn get_property(
        &self,
        sc: &mut crate::local::LocalScope,
        key: crate::value::object::PropertyKey,
    ) -> Result<Value, Value> {
        self.obj.get_property(sc, key)
    }

    fn get_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
    ) -> Result<Option<super::object::PropertyValue>, Value> {
        self.obj.get_property_descriptor(sc, key)
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
        self.obj.own_keys()
    }
}

#[derive(Debug, Trace)]
pub struct PromiseResolver {
    promise: Handle<dyn Object>,
    obj: NamedObject,
}

impl PromiseResolver {
    pub fn new(vm: &mut Vm, promise: Handle<dyn Object>) -> Self {
        Self {
            promise,
            obj: NamedObject::new(vm),
        }
    }
}

impl Object for PromiseResolver {
    fn get_property(
        &self,
        sc: &mut crate::local::LocalScope,
        key: crate::value::object::PropertyKey,
    ) -> Result<Value, Value> {
        self.obj.get_property(sc, key)
    }

    fn get_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
    ) -> Result<Option<super::object::PropertyValue>, Value> {
        self.obj.get_property_descriptor(sc, key)
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
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        scope.drive_promise(
            PromiseAction::Resolve,
            self.promise.as_any().downcast_ref::<Promise>().unwrap(),
            args,
        );

        Ok(Value::undefined())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.obj.own_keys()
    }

    fn type_of(&self) -> super::Typeof {
        Typeof::Function
    }
}

#[derive(Debug, Trace)]
pub struct PromiseRejecter {
    promise: Handle<dyn Object>,
    obj: NamedObject,
}

impl PromiseRejecter {
    pub fn new(vm: &mut Vm, promise: Handle<dyn Object>) -> Self {
        Self {
            promise,
            obj: NamedObject::new(vm),
        }
    }
}

impl Object for PromiseRejecter {
    fn get_property(
        &self,
        sc: &mut crate::local::LocalScope,
        key: crate::value::object::PropertyKey,
    ) -> Result<Value, Value> {
        self.obj.get_property(sc, key)
    }

    fn get_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
    ) -> Result<Option<super::object::PropertyValue>, Value> {
        self.obj.get_property_descriptor(sc, key)
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
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        scope.drive_promise(
            PromiseAction::Reject,
            self.promise.as_any().downcast_ref::<Promise>().unwrap(),
            args,
        );

        Ok(Value::undefined())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.obj.own_keys()
    }

    fn type_of(&self) -> super::Typeof {
        Typeof::Function
    }
}

/// Wraps the passed value in a resolved promise, unless it already is a promise
pub fn wrap_promise(scope: &mut LocalScope, value: Value) -> Value {
    if let Value::Object(object) = &value {
        if object.as_any().is::<Promise>() {
            return value.clone();
        }
    }

    let promise = Promise::resolved(scope, value.clone());
    Value::Object(scope.register(promise))
}
