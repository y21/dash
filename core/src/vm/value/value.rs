use core::fmt::Debug;
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use crate::vm::VM;

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
    pub fields: HashMap<Box<str>, Rc<RefCell<Value>>>,
    /// [[Prototype]] of this value
    pub proto: Option<Weak<RefCell<Value>>>,
    /// Constructor of this value
    pub constructor: Option<Weak<RefCell<Value>>>,
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

    /// Updates the internal properties ([[Prototype]] and constructor)
    /// of this JavaScript value
    pub fn update_internal_properties(
        &mut self,
        proto: &Rc<RefCell<Value>>,
        ctor: &Rc<RefCell<Value>>,
    ) {
        self.proto = Some(Rc::downgrade(proto));
        self.constructor = Some(Rc::downgrade(ctor));
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
                    Object::Promise(_) => self
                        .update_internal_properties(&statics.promise_proto, &statics.promise_ctor),
                    Object::String(_) => {
                        self.update_internal_properties(&statics.string_proto, &statics.string_ctor)
                    }
                    Object::Function(_) => self.update_internal_properties(
                        &statics.function_proto,
                        &statics.function_ctor,
                    ),
                    Object::Array(_) => {
                        self.update_internal_properties(&statics.array_proto, &statics.array_ctor)
                    }
                    Object::Any(_) => {
                        self.update_internal_properties(&statics.object_proto, &statics.object_ctor)
                    }
                    Object::Weak(JsWeak::Set(_)) => self
                        .update_internal_properties(&statics.weakset_proto, &statics.weakset_ctor),
                    Object::Weak(JsWeak::Map(_)) => self
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
            ValueKind::Object(o) => match &**o {
                Object::String(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    /// Returns a Rc to the [[Prototype]] of this value, if it has one
    pub fn strong_proto(&self) -> Option<Rc<RefCell<Value>>> {
        self.proto.as_ref().and_then(Weak::upgrade)
    }

    /// Returns a Rc to the constructor of this value, if it has one
    pub fn strong_constructor(&self) -> Option<Rc<RefCell<Value>>> {
        self.constructor.as_ref().and_then(Weak::upgrade)
    }

    /// Tries to unwrap a Rc<RefCell<Value>> into a Value
    pub fn try_into_inner(value: Rc<RefCell<Self>>) -> Option<Self> {
        Some(Rc::try_unwrap(value).ok()?.into_inner())
    }

    /// Unwraps o, or returns undefined if it is None
    pub fn unwrap_or_undefined(o: Option<Rc<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        o.unwrap_or_else(|| Value::new(ValueKind::Undefined).into())
    }

    /// Looks up a field directly
    pub fn get_field(&self, key: &str) -> Option<&Rc<RefCell<Value>>> {
        self.fields.get(key)
    }

    /// Looks up a property and goes through exotic property matching
    ///
    /// For a direct field lookup, use [Value::get_field]
    pub fn get_property(
        vm: &VM,
        value_cell: &Rc<RefCell<Value>>,
        key: &str,
        override_this: Option<&Rc<RefCell<Value>>>,
    ) -> Option<Rc<RefCell<Value>>> {
        let value = value_cell.borrow();
        let key = key.into();

        match key {
            "__proto__" => {
                return Some(
                    value
                        .strong_proto()
                        .unwrap_or_else(|| Value::new(ValueKind::Null).into()),
                )
            }
            "constructor" => return value.strong_constructor(),
            "prototype" => {
                if let Some(func) = value.as_function() {
                    return func.prototype();
                }
            }
            "length" => {
                match value.as_object() {
                    Some(Object::Array(a)) => {
                        return Some(vm.create_js_value(a.elements.len() as f64).into())
                    }
                    Some(Object::String(s)) => {
                        return Some(vm.create_js_value(s.len() as f64).into())
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

        if value.fields.len() > 0 {
            if let Some(entry_cell) = value.fields.get(key) {
                if let Some(override_this) = override_this {
                    let mut entry = entry_cell.borrow_mut();

                    if let Some(f) = entry.as_function_mut() {
                        let receiver = Receiver::Bound(Rc::clone(&override_this));

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
                return Some(Rc::clone(entry_cell));
            }
        }

        if let Some(proto_cell) = value.proto.as_ref().and_then(Weak::upgrade) {
            Value::get_property(
                vm,
                &proto_cell,
                key,
                override_this.or_else(|| Some(value_cell)),
            )
        } else {
            None
        }
    }

    /// Adds a field
    pub fn set_property(&mut self, k: impl Into<Box<str>>, v: Rc<RefCell<Value>>) {
        self.fields.insert(k.into(), v);
    }
}
