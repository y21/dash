use core::fmt::Debug;
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use crate::{
    gc::Handle,
    js_std,
    vm::{
        frame::Frame,
        value::{function::CallContext, object::ExoticObject},
        VM,
    },
};

use super::{
    function::{FunctionKind, Receiver},
    object::{Object, Weak as JsWeak},
    ValueKind,
};

/// A wrapper for Rc<T>, but always implements the Hash trait by hasing the pointer
///
/// This makes it suitable for putting JavaScript values in a HashMap
#[derive(Debug, Clone)]
pub struct HashRc<T>(pub Rc<T>);

/// A wrapper for Weak<T>, but always implements the Hash trait by hasing the pointer
///
/// This makes it suitable for putting weak JavaScript values in a HashMap
#[derive(Debug, Clone)]
pub struct HashWeak<T>(pub Weak<T>);

impl<T> Hash for HashRc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state)
    }
}
impl<T> PartialEq for HashRc<T> {
    fn eq(&self, other: &HashRc<T>) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for HashRc<T> {}

impl<T> Hash for HashWeak<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Weak::as_ptr(&self.0).hash(state)
    }
}
impl<T> PartialEq for HashWeak<T> {
    fn eq(&self, other: &HashWeak<T>) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for HashWeak<T> {}

/// A JavaScript value
#[derive(Debug, Clone)]
pub struct Value {
    /// The type of value
    pub kind: ValueKind,
    /// The fields of this value
    pub fields: HashMap<Box<str>, Handle<Value>>,
    /// [[Prototype]] of this value
    pub proto: Option<Handle<Value>>,
    /// Constructor of this value
    pub constructor: Option<Handle<Value>>,
}

impl Value {
    /// Creates a new value
    ///
    /// It is recommended to only create values using this function
    /// if it is not necessary to have a [[Prototype]] set, such as for
    /// undefined and null values
    pub fn new(kind: ValueKind) -> Self {
        Self {
            kind,
            fields: HashMap::new(),
            constructor: None,
            proto: None,
        }
    }

    /// Attempts to call a value
    pub fn call(
        this: &Handle<Value>,
        mut args: Vec<Handle<Value>>,
        vm: &mut VM,
    ) -> Result<Handle<Value>, Handle<Value>> {
        let value = unsafe { this.borrow_unbounded() };

        let func = match value.as_function() {
            Some(FunctionKind::Native(func)) => {
                let receiver = func.receiver.as_ref().map(|rx| rx.get().clone());
                let ctx = CallContext {
                    vm,
                    args: &mut args,
                    ctor: false,
                    receiver,
                };

                return (func.func)(ctx);
            }
            Some(FunctionKind::Closure(closure)) => &closure.func,
            None => {
                return Err(js_std::error::create_error(
                    "Invoked value is not a function".into(),
                    vm,
                ))
            }
            _ => unreachable!(),
        };

        let sp = vm.stack.len();

        let frame = Frame {
            ip: 0,
            func: Handle::clone(this),
            buffer: func.buffer.clone(),
            sp,
        };

        let origin_param_count = func.params as usize;
        let param_count = args.len();

        for param in args.into_iter() {
            vm.stack.push(param);
        }

        for _ in 0..(origin_param_count.saturating_sub(param_count)) {
            vm.stack
                .push(Value::new(ValueKind::Undefined).into_handle(vm));
        }

        match vm.execute_frame(frame, true) {
            Ok(Some(ret)) => Ok(ret),
            Ok(None) => Ok(Value::new(ValueKind::Undefined).into_handle(vm)),
            Err(e) => Err(e.into_value()),
        }
    }

    /// Registers this value for garbage collection and returns a handle to it
    // TODO: re-think whether this is fine to not be unsafe?
    pub fn into_handle(self, vm: &VM) -> Handle<Self> {
        vm.gc.borrow_mut().register(self)
    }

    /// Updates the internal properties ([[Prototype]] and constructor)
    /// of this JavaScript value
    pub fn update_internal_properties(&mut self, proto: &Handle<Value>, ctor: &Handle<Value>) {
        self.proto = Some(Handle::clone(proto));
        self.constructor = Some(Handle::clone(ctor));
    }

    /// Tries to detect the [[Prototype]] and constructor of this value given self.kind, and updates it
    pub fn detect_internal_properties(&mut self, vm: &VM) {
        let statics = &vm.statics;
        match &self.kind {
            ValueKind::Bool(_) => {
                self.update_internal_properties(&statics.boolean_proto, &statics.boolean_ctor)
            }
            ValueKind::Number(_) => {
                self.update_internal_properties(&statics.number_proto, &statics.number_ctor)
            }
            ValueKind::Object(o) => {
                // can't pattern match box ;/
                match &**o {
                    Object::Exotic(ExoticObject::Promise(_)) => self
                        .update_internal_properties(&statics.promise_proto, &statics.promise_ctor),
                    Object::Exotic(ExoticObject::String(_)) => {
                        self.update_internal_properties(&statics.string_proto, &statics.string_ctor)
                    }
                    Object::Exotic(ExoticObject::Function(_)) => self.update_internal_properties(
                        &statics.function_proto,
                        &statics.function_ctor,
                    ),
                    Object::Exotic(ExoticObject::Array(_)) => {
                        self.update_internal_properties(&statics.array_proto, &statics.array_ctor)
                    }
                    Object::Ordinary => {
                        self.update_internal_properties(&statics.object_proto, &statics.object_ctor)
                    }
                    Object::Exotic(ExoticObject::Weak(JsWeak::Set(_))) => self
                        .update_internal_properties(&statics.weakset_proto, &statics.weakset_ctor),
                    Object::Exotic(ExoticObject::Weak(JsWeak::Map(_))) => self
                        .update_internal_properties(&statics.weakmap_proto, &statics.weakmap_ctor),
                };
            }
            _ => {}
        }
    }

    /// Returns whether this value is a primitive
    pub fn is_primitive(&self) -> bool {
        // https://262.ecma-international.org/6.0/#sec-toprimitive
        match &self.kind {
            ValueKind::Number(_) => true,
            ValueKind::Bool(_) => true,
            ValueKind::Null => true,
            ValueKind::Undefined => true,
            ValueKind::Object(o) => matches!(&**o, Object::Exotic(ExoticObject::String(_))),
        }
    }

    /// Returns whether this value is callable
    pub fn is_callable(&self) -> bool {
        match &self.kind {
            ValueKind::Object(o) => matches!(&**o, Object::Exotic(ExoticObject::Function(_))),
            _ => false,
        }
    }

    /// Returns a Rc to the [[Prototype]] of this value, if it has one
    pub fn strong_proto(&self) -> Option<Handle<Value>> {
        self.proto.clone()
    }

    /// Returns a Rc to the constructor of this value, if it has one
    pub fn strong_constructor(&self) -> Option<Handle<Value>> {
        self.constructor.clone()
    }

    /// Tries to unwrap a Handle<Value> into a Value
    pub fn try_into_inner(value: Rc<RefCell<Self>>) -> Option<Self> {
        Some(Rc::try_unwrap(value).ok()?.into_inner())
    }

    /// Unwraps o, or returns undefined if it is None
    pub fn unwrap_or_undefined(o: Option<Handle<Self>>, vm: &VM) -> Handle<Self> {
        o.unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(vm))
    }

    /// Looks up a field directly
    pub fn get_field(&self, key: &str) -> Option<&Handle<Value>> {
        self.fields.get(key)
    }

    /// Looks up a property and goes through exotic property matching
    ///
    /// For a direct field lookup, use [Value::get_field]
    pub fn get_property(
        vm: &VM,
        value_cell: &Handle<Value>,
        key: &str,
        override_this: Option<&Handle<Value>>,
    ) -> Option<Handle<Value>> {
        let value = unsafe { value_cell.borrow_unbounded() };

        match key {
            "__proto__" => {
                return Some(
                    value
                        .strong_proto()
                        .unwrap_or_else(|| Value::new(ValueKind::Null).into_handle(vm)),
                )
            }
            "constructor" => return value.strong_constructor(),
            "prototype" => {
                if let Some(func) = value.as_function() {
                    return func.prototype().cloned();
                }
            }
            "length" => {
                match value.as_object() {
                    Some(Object::Exotic(ExoticObject::Array(a))) => {
                        return Some(vm.create_js_value(a.elements.len() as f64).into_handle(vm))
                    }
                    Some(Object::Exotic(ExoticObject::String(s))) => {
                        return Some(vm.create_js_value(s.len() as f64).into_handle(vm))
                    }
                    _ => {}
                };
            }
            _ => {
                if let Ok(idx) = key.parse::<usize>() {
                    if let Some(a) = value.as_object().and_then(Object::as_array) {
                        return a.elements.get(idx).cloned();
                    }
                }
            }
        };

        if !value.fields.is_empty() {
            if let Some(entry_cell) = value.fields.get(key) {
                if let Some(override_this) = override_this {
                    let mut entry = unsafe { entry_cell.borrow_mut_unbounded() };

                    if let Some(f) = entry.as_function_mut() {
                        let receiver = Receiver::Bound(Handle::clone(&override_this));

                        match f {
                            FunctionKind::Closure(c) => c.func.bind(receiver),
                            FunctionKind::Native(n) => {
                                if let Some(recv) = &mut n.receiver {
                                    recv.bind(receiver);
                                } else {
                                    n.receiver = Some(receiver);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                return Some(Handle::clone(entry_cell));
            }
        }

        if let Some(proto_cell) = value.proto.as_ref() {
            Value::get_property(vm, proto_cell, key, override_this.or(Some(value_cell)))
        } else {
            None
        }
    }

    /// Adds a field
    pub fn set_property(&mut self, k: impl Into<Box<str>>, v: Handle<Value>) {
        self.fields.insert(k.into(), v);
    }

    pub(crate) fn mark(this: &Handle<Value>) {
        let mut this = if let Ok(this) = unsafe { this.get_unchecked().try_borrow_mut() } {
            this
        } else {
            return;
        };

        if this.is_marked() {
            // We're already marked as visited. Don't get stuck in an infinite loop
            return;
        }

        this.mark_visited();

        if let Some(proto) = &this.proto {
            Value::mark(proto)
        }

        if let Some(constructor) = &this.constructor {
            Value::mark(constructor)
        }

        for handle in this.fields.values() {
            Value::mark(handle)
        }

        match &this.kind {
            ValueKind::Object(o) => match &**o {
                Object::Exotic(ExoticObject::Array(a)) => {
                    for handle in &a.elements {
                        Value::mark(handle)
                    }
                }
                Object::Exotic(ExoticObject::Function(f)) => f.mark(),
                Object::Exotic(ExoticObject::Promise(_)) => todo!(),
                _ => {}
            },
            _ => {}
        };
    }
}
