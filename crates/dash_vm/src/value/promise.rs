use std::cell::RefCell;

use dash_proc_macro::Trace;

use crate::gc::ObjectId;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::localscope::LocalScope;
use crate::value::object::This;
use crate::{PromiseAction, Vm, extract};

use super::function::args::CallArgs;
use super::object::{Object, OrdObject, PropertyValue};
use super::propertykey::PropertyKey;
use super::{Typeof, Unrooted, Value};

#[derive(Debug)]
pub enum PromiseState {
    Pending {
        resolve: Vec<ObjectId>,
        reject: Vec<ObjectId>,
    },
    Resolved(Value),
    Rejected {
        value: Value,
        caught: bool,
    },
}

unsafe impl Trace for PromiseState {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            Self::Pending { resolve, reject } => {
                resolve.trace(cx);
                reject.trace(cx);
            }
            Self::Resolved(v) => v.trace(cx),
            Self::Rejected { value, caught: _ } => value.trace(cx),
        }
    }
}

#[derive(Debug, Trace)]
pub struct Promise {
    state: RefCell<PromiseState>,
    obj: OrdObject,
}

impl Promise {
    pub fn new(vm: &Vm) -> Self {
        Self::with_obj(OrdObject::with_prototype(vm.statics.promise_proto))
    }

    pub fn with_obj(obj: OrdObject) -> Self {
        Self {
            state: RefCell::new(PromiseState::Pending {
                reject: Vec::new(),
                resolve: Vec::new(),
            }),
            obj,
        }
    }

    pub fn resolved(vm: &Vm, value: Value) -> Self {
        Self {
            state: RefCell::new(PromiseState::Resolved(value)),
            obj: OrdObject::with_prototype(vm.statics.promise_proto),
        }
    }

    pub fn rejected(scope: &mut LocalScope, value: Value) -> ObjectId {
        let this = scope.register(Self {
            state: RefCell::new(PromiseState::Rejected { value, caught: false }),
            obj: OrdObject::with_prototype(scope.statics.promise_proto),
        });
        scope.rejected_promises.insert(this);
        this
    }

    pub fn state(&self) -> &RefCell<PromiseState> {
        &self.state
    }
}

impl Object for Promise {
    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        self.obj.get_own_property_descriptor(key, sc)
    }

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope) -> Result<(), Value> {
        self.obj.set_property(key, value, sc)
    }

    fn delete_property(&self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Value> {
        self.obj.delete_property(key, sc)
    }

    fn set_prototype(&self, value: Value, sc: &mut LocalScope) -> Result<(), Value> {
        self.obj.set_prototype(value, sc)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        callee: ObjectId,
        this: This,
        args: CallArgs,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(callee, this, args, scope)
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.obj.own_keys(sc)
    }

    extract!(self);
}

#[derive(Debug, Trace)]
pub struct PromiseResolver {
    promise: ObjectId,
    obj: OrdObject,
}

impl PromiseResolver {
    pub fn new(vm: &Vm, promise: ObjectId) -> Self {
        Self {
            promise,
            obj: OrdObject::new(vm),
        }
    }
}

impl Object for PromiseResolver {
    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        self.obj.get_own_property_descriptor(key, sc)
    }

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope) -> Result<(), Value> {
        self.obj.set_property(key, value, sc)
    }

    fn delete_property(&self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Value> {
        self.obj.delete_property(key, sc)
    }

    fn set_prototype(&self, value: Value, sc: &mut LocalScope) -> Result<(), Value> {
        self.obj.set_prototype(value, sc)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        _callee: ObjectId,
        _this: This,
        args: CallArgs,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        scope.drive_promise(
            PromiseAction::Resolve,
            self.promise.extract::<Promise>(scope).unwrap(),
            self.promise,
            args,
        );

        Ok(Value::undefined().into())
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.obj.own_keys(sc)
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Function
    }

    extract!(self);
}

#[derive(Debug, Trace)]
pub struct PromiseRejecter {
    promise: ObjectId,
    obj: OrdObject,
}

impl PromiseRejecter {
    pub fn new(vm: &Vm, promise: ObjectId) -> Self {
        Self {
            promise,
            obj: OrdObject::new(vm),
        }
    }
}

impl Object for PromiseRejecter {
    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        self.obj.get_own_property_descriptor(key, sc)
    }

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope) -> Result<(), Value> {
        self.obj.set_property(key, value, sc)
    }

    fn delete_property(&self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Value> {
        self.obj.delete_property(key, sc)
    }

    fn set_prototype(&self, value: Value, sc: &mut LocalScope) -> Result<(), Value> {
        self.obj.set_prototype(value, sc)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        _callee: ObjectId,
        _this: This,
        args: CallArgs,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        scope.drive_promise(
            PromiseAction::Reject,
            self.promise.extract::<Promise>(scope).unwrap(),
            self.promise,
            args,
        );

        Ok(Value::undefined().into())
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.obj.own_keys(sc)
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Function
    }

    extract!(self);
}

/// Wraps the passed value in a resolved promise, unless it already is a promise
pub fn wrap_resolved_promise(scope: &mut LocalScope, value: Value) -> Value {
    if value.extract::<Promise>(scope).is_some() {
        return value;
    }

    let promise = Promise::resolved(scope, value);
    Value::object(scope.register(promise))
}
