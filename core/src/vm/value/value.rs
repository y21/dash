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
    weak::MaybeWeak,
    ValueKind,
};

#[derive(Debug, Clone)]
pub struct HashRc<T>(pub Rc<T>);

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

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub fields: HashMap<Box<str>, Rc<RefCell<Value>>>,
    /// [[Prototype]] of this value
    pub proto: Option<Weak<RefCell<Value>>>,
    /// Constructor of this value
    pub constructor: Option<Weak<RefCell<Value>>>,
}

impl Value {
    pub fn new(kind: ValueKind) -> Self {
        Self {
            kind,
            fields: HashMap::new(),
            constructor: None,
            proto: None,
        }
    }

    pub fn with_prototype(kind: ValueKind, proto: MaybeWeak<RefCell<Value>>) -> Self {
        Self {
            kind,
            fields: HashMap::new(),
            constructor: None,
            proto: Some(proto.into_weak()),
        }
    }

    pub fn with_constructor(kind: ValueKind, constructor: MaybeWeak<RefCell<Value>>) -> Self {
        Self {
            kind,
            fields: HashMap::new(),
            constructor: Some(constructor.into_weak()),
            proto: None,
        }
    }

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

    pub fn strong_proto(&self) -> Option<Rc<RefCell<Value>>> {
        self.proto.as_ref().and_then(Weak::upgrade)
    }

    pub fn strong_constructor(&self) -> Option<Rc<RefCell<Value>>> {
        self.constructor.as_ref().and_then(Weak::upgrade)
    }

    pub fn try_into_inner(value: Rc<RefCell<Self>>) -> Option<Self> {
        Some(Rc::try_unwrap(value).ok()?.into_inner())
    }

    pub fn unwrap_or_undefined(o: Option<Rc<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        o.unwrap_or_else(|| Value::new(ValueKind::Undefined).into())
    }

    pub fn get_field(&self, key: &str) -> Option<&Rc<RefCell<Value>>> {
        self.fields.get(key)
    }

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
                match &value.kind {
                    ValueKind::Object(o) => match &**o {
                        Object::Array(a) => {
                            return Some(vm.create_js_value(a.elements.len() as f64).into())
                        }
                        Object::String(s) => {
                            return Some(vm.create_js_value(s.len() as f64).into())
                        }
                        _ => {}
                    },
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
            Value::get_property(vm, &proto_cell, key, Some(value_cell))
        } else {
            None
        }
    }

    pub fn set_property(&mut self, k: impl Into<Box<str>>, v: Rc<RefCell<Value>>) {
        self.fields.insert(k.into(), v);
    }
}
