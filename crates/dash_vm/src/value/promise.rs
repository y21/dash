use std::any::Any;
use std::cell::RefCell;

use dash_proc_macro::Trace;

use crate::gc::trace::{Trace, TraceCtxt};
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::{PromiseAction, Vm};

use super::object::{NamedObject, Object};
use super::{Typeof, Unpack, Unrooted, Value, ValueKind};

#[derive(Debug)]
pub enum PromiseState {
    Pending {
        resolve: Vec<ObjectId>,
        reject: Vec<ObjectId>,
    },
    Resolved(Value),
    Rejected(Value),
}

unsafe impl Trace for PromiseState {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            Self::Pending { resolve, reject } => {
                resolve.trace(cx);
                reject.trace(cx);
            }
            Self::Resolved(v) => v.trace(cx),
            Self::Rejected(v) => v.trace(cx),
        }
    }
}

#[derive(Debug, Trace)]
pub struct Promise {
    state: RefCell<PromiseState>,
    obj: NamedObject,
}

impl Promise {
    pub fn new(vm: &Vm) -> Self {
        Self {
            state: RefCell::new(PromiseState::Pending {
                reject: Vec::new(),
                resolve: Vec::new(),
            }),
            obj: NamedObject::with_prototype_and_constructor(
                vm.statics.promise_proto,
                vm.statics.promise_ctor,
            ),
        }
    }
    pub fn resolved(vm: &Vm, value: Value) -> Self {
        Self {
            state: RefCell::new(PromiseState::Resolved(value)),
            obj: NamedObject::with_prototype_and_constructor(vm.statics.promise_proto, vm.statics.promise_ctor),
        }
    }
    pub fn rejected(vm: &Vm, value: Value) -> Self {
        Self {
            state: RefCell::new(PromiseState::Rejected(value)),
            obj: NamedObject::with_prototype_and_constructor(vm.statics.promise_proto, vm.statics.promise_ctor),
        }
    }
    pub fn state(&self) -> &RefCell<PromiseState> {
        &self.state
    }
}

impl Object for Promise {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
    ) -> Result<Option<super::object::PropertyValue>, Unrooted> {
        self.obj.get_own_property_descriptor(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut crate::localscope::LocalScope,
        key: crate::value::object::PropertyKey,
        value: crate::value::object::PropertyValue,
    ) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(
        &self,
        sc: &mut crate::localscope::LocalScope,
        key: crate::value::object::PropertyKey,
    ) -> Result<Unrooted, Value> {
        self.obj.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut crate::localscope::LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut crate::localscope::LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        scope: &mut crate::localscope::LocalScope,
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self, _: &Vm) -> &dyn Any {
        self
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.obj.own_keys(sc)
    }
}

#[derive(Debug, Trace)]
pub struct PromiseResolver {
    promise: ObjectId,
    obj: NamedObject,
}

impl PromiseResolver {
    pub fn new(vm: &Vm, promise: ObjectId) -> Self {
        Self {
            promise,
            obj: NamedObject::new(vm),
        }
    }
}

impl Object for PromiseResolver {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
    ) -> Result<Option<super::object::PropertyValue>, Unrooted> {
        self.obj.get_own_property_descriptor(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut crate::localscope::LocalScope,
        key: crate::value::object::PropertyKey,
        value: crate::value::object::PropertyValue,
    ) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(
        &self,
        sc: &mut crate::localscope::LocalScope,
        key: crate::value::object::PropertyKey,
    ) -> Result<Unrooted, Value> {
        self.obj.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut crate::localscope::LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut crate::localscope::LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        scope: &mut crate::localscope::LocalScope,
        _callee: ObjectId,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        scope.drive_promise(
            PromiseAction::Resolve,
            self.promise.as_any(scope).downcast_ref::<Promise>().unwrap(),
            args,
        );

        Ok(Value::undefined().into())
    }

    fn as_any(&self, _: &Vm) -> &dyn Any {
        self
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.obj.own_keys(sc)
    }

    fn type_of(&self, _: &Vm) -> super::Typeof {
        Typeof::Function
    }
}

#[derive(Debug, Trace)]
pub struct PromiseRejecter {
    promise: ObjectId,
    obj: NamedObject,
}

impl PromiseRejecter {
    pub fn new(vm: &Vm, promise: ObjectId) -> Self {
        Self {
            promise,
            obj: NamedObject::new(vm),
        }
    }
}

impl Object for PromiseRejecter {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
    ) -> Result<Option<super::object::PropertyValue>, Unrooted> {
        self.obj.get_own_property_descriptor(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut crate::localscope::LocalScope,
        key: crate::value::object::PropertyKey,
        value: crate::value::object::PropertyValue,
    ) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(
        &self,
        sc: &mut crate::localscope::LocalScope,
        key: crate::value::object::PropertyKey,
    ) -> Result<Unrooted, Value> {
        self.obj.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut crate::localscope::LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut crate::localscope::LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        scope: &mut crate::localscope::LocalScope,
        _callee: ObjectId,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        scope.drive_promise(
            PromiseAction::Reject,
            self.promise.as_any(scope).downcast_ref::<Promise>().unwrap(),
            args,
        );

        Ok(Value::undefined().into())
    }

    fn as_any(&self, _: &Vm) -> &dyn Any {
        self
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.obj.own_keys(sc)
    }

    fn type_of(&self, _: &Vm) -> super::Typeof {
        Typeof::Function
    }
}

/// Wraps the passed value in a resolved promise, unless it already is a promise
pub fn wrap_promise(scope: &mut LocalScope, value: Value) -> Value {
    if let ValueKind::Object(object) = value.unpack() {
        if object.as_any(scope).is::<Promise>() {
            return value;
        }
    }

    let promise = Promise::resolved(scope, value);
    Value::object(scope.register(promise))
}
