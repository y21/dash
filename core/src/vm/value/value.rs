use core::fmt::Debug;
use std::{
    borrow::{BorrowMut, Cow},
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use crate::{
    gc::Handle,
    js_std,
    vm::{
        dispatch::DispatchResult,
        frame::Frame,
        value::{function::CallContext, object::ExoticObject},
        VM,
    },
};

use super::{
    function::{FunctionKind, Receiver},
    object::{Object, ObjectKind},
    ops::logic::Typeof,
    weak::Weak as JsWeak,
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

/// A property key
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PropertyKey<'a> {
    /// String
    String(Cow<'a, str>),
    /// Symbol
    Symbol(Handle<Object>),
}

impl<'a> PropertyKey<'a> {
    /// Returns the inner string of this property, if it is a string
    pub fn as_str(&self) -> Option<&Cow<'a, str>> {
        match self {
            PropertyKey::String(s) => Some(s),
            PropertyKey::Symbol(_) => None,
        }
    }

    /// Inspects this property key
    pub fn inspect(&self, vm: &VM, depth: u32) -> String {
        match self {
            PropertyKey::String(s) => s.to_string(),
            PropertyKey::Symbol(s) => {
                let s = unsafe { s.borrow_unbounded() };
                s.inspect(vm, depth).to_string()
            }
        }
    }

    pub(crate) fn mark(&self) {
        if let PropertyKey::Symbol(handle) = self {
            let mut handle = unsafe { handle.get_unchecked().borrow_mut() };
            handle.mark_visited();
        }
    }
}

impl From<String> for PropertyKey<'_> {
    fn from(s: String) -> Self {
        Self::String(Cow::Owned(s))
    }
}

impl<'a> From<&'a str> for PropertyKey<'a> {
    fn from(s: &'a str) -> Self {
        Self::String(Cow::Borrowed(s))
    }
}

impl From<Handle<Object>> for PropertyKey<'_> {
    fn from(h: Handle<Object>) -> Self {
        Self::Symbol(h)
    }
}

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
}

impl Value {
    /// Creates a new value
    pub fn new(kind: ValueKind) -> Self {
        Self { kind }
    }

    /// Attempts to call a value
    pub fn call(&self, mut args: Vec<Value>, vm: &mut VM) -> Result<Value, Value> {
        let obj = self
            .as_object()
            .ok_or_else(|| js_std::error::create_error("Value is not a function", vm))?;

        assert!(obj.check_marker(vm));

        let value = unsafe { obj.borrow_unbounded() };

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
                    "Invoked value is not a function",
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
            iterator_caller: None,
            is_constructor: false,
        };

        let origin_param_count = func.params as usize;
        let param_count = args.len();

        for param in args.into_iter() {
            vm.try_push_stack(param)?;
        }

        for _ in 0..(origin_param_count.saturating_sub(param_count)) {
            vm.stack.push(Value::new(ValueKind::Undefined));
        }

        match vm.execute_frame(frame, false) {
            Ok(DispatchResult::Return(Some(r)) | DispatchResult::Yield(Some(r))) => Ok(r),
            Ok(_) => Ok(Value::new(ValueKind::Undefined)),
            Err(e) => Err(e.into_value()),
        }
    }

    // /// Registers this value for garbage collection and returns a handle to it
    // // TODO: re-think whether this is fine to not be unsafe?
    // pub fn into_handle(self, vm: &VM) -> Handle<Self> {
    //     vm.gc.borrow_mut().register(self)
    // }

    /// Updates the internal properties ([[Prototype]] and constructor)
    /// of this JavaScript value
    pub fn update_internal_properties(
        &self,
        vm: &VM,
        proto: &Handle<Object>,
        ctor: &Handle<Object>,
    ) {
        self.as_object()
            .map(|x| x.borrow_mut(vm).update_internal_properties(proto, ctor));
    }

    /// Updates the [[Prototype]] of this JavaScript value
    pub fn set_prototype(&self, vm: &VM, proto: Option<&Handle<Object>>) {
        if let ValueKind::Object(obj) = &self.kind {
            obj.borrow_mut(vm).prototype = proto.cloned();
        }
    }

    /// Tries to detect the [[Prototype]] and constructor of this value given self.kind, and updates it
    pub fn detect_internal_properties(&self, vm: &VM) {
        if let Some(object) = self.as_object().map(|x| x.borrow_mut(vm)) {
            object.detect_internal_properties(vm);
        }
    }

    /// Returns whether this value is a primitive
    pub fn is_primitive(&self, vm: &VM) -> bool {
        // https://262.ecma-international.org/6.0/#sec-toprimitive
        match &self.kind {
            ValueKind::Number(_) => true,
            ValueKind::Bool(_) => true,
            ValueKind::Null => true,
            ValueKind::Undefined => true,
            ValueKind::Object(o) => o.borrow(vm).is_primitive(),
        }
    }

    /// Returns whether this value is callable
    pub fn is_callable(&self, vm: &VM) -> bool {
        self.as_object()
            .map(|x| x.borrow(vm).is_callable())
            .unwrap_or(false)
    }

    /// Checks whether this value is strictly a function
    pub fn is_function(&self) -> bool {
        self._typeof() == Typeof::Function
    }

    /// Returns a reference to the [[Prototype]] of this value, if it has one
    pub fn prototype(&self, vm: &VM) -> Option<Handle<Object>> {
        match &self.kind {
            ValueKind::Bool(_) => Some(Handle::clone(&vm.statics.boolean_proto)),
            ValueKind::Number(_) => Some(Handle::clone(&vm.statics.number_proto)),
            ValueKind::Null | ValueKind::Undefined => None,
            ValueKind::Object(o) => o.borrow(vm).prototype.as_ref().cloned(),
        }
    }

    /// Returns a reference to the inner [[Prototype]] of this value if it is an object
    ///
    /// The prototype of primitive values never changes
    pub fn object_prototype(&self, vm: &VM) -> Option<Handle<Object>> {
        self.as_object()
            .and_then(|o| o.borrow(vm).prototype.as_ref().cloned())
    }

    /// Returns a reference to the constructor of this value, if it has one
    pub fn constructor(&self, vm: &VM) -> Option<Handle<Object>> {
        match &self.kind {
            ValueKind::Bool(_) => Some(Handle::clone(&vm.statics.boolean_ctor)),
            ValueKind::Number(_) => Some(Handle::clone(&vm.statics.number_ctor)),
            ValueKind::Null | ValueKind::Undefined => None,
            ValueKind::Object(o) => o.borrow(vm).constructor.as_ref().cloned(),
            _ => None,
        }
    }

    /// Returns a reference to the inner constructor of this value if it is an object
    ///
    /// The constructor of primitive values never changes
    pub fn object_constructor(&self, vm: &VM) -> Option<Handle<Object>> {
        self.as_object()
            .and_then(|o| o.borrow(vm).constructor.as_ref().cloned())
    }

    /// Unwraps o, or returns undefined if it is None
    pub fn unwrap_or_undefined(o: Option<Self>, vm: &VM) -> Self {
        o.unwrap_or_else(|| Value::new(ValueKind::Undefined))
    }

    /// Looks up a field directly without going up the prototype chain
    pub fn get_field(&self, key: PropertyKey<'_>) -> Option<Handle<Value>> {
        self.fields().and_then(|x| x.get(&key).cloned())
    }

    /// Checks whether this value contains a particular key without walking the prototype chain
    pub fn has_field(&self, key: PropertyKey<'_>) -> bool {
        self.fields().map(|x| x.contains_key(&key)).unwrap_or(false)
    }

    /// Checks whether this value (or one of the values in its prototype chain) contains a field
    pub fn has_property(&self, vm: &VM, key: PropertyKey<'_>) -> bool {
        if self.has_field(key.clone()) {
            return true;
        }

        if let Some(proto) = self.prototype(vm).as_ref() {
            let proto_ref = proto.borrow(vm);
            proto_ref.has_property(vm, key)
        } else {
            false
        }
    }

    /// Returns a reference to the inner HashMap of JS values
    pub fn fields(&self) -> Option<&HashMap<PropertyKey<'static>, Handle<Value>>> {
        match &self.kind {
            ValueKind::Object(o) => Some(&o.fields),
            _ => None,
        }
    }

    /// Returns a reference to the inner HashMap of JS values
    pub fn fields_mut(&mut self) -> Option<&mut HashMap<PropertyKey<'static>, Handle<Value>>> {
        match &mut self.kind {
            ValueKind::Object(o) => Some(&mut o.fields),
            _ => None,
        }
    }

    /// Looks up a property and goes through exotic property matching
    ///
    /// For a direct field lookup, use [Value::get_field]
    pub fn get_property(
        vm: &VM,
        value_cell: &Handle<Value>,
        key: &PropertyKey<'_>,
        override_this: Option<&Handle<Value>>,
    ) -> Option<Handle<Value>> {
        let value = unsafe { value_cell.borrow_unbounded() };

        // TODO: refactor this with Exotic trait
        match key.as_str().map(|x| x.as_ref()) {
            Some("__proto__") => {
                return Some(
                    value
                        .prototype(vm)
                        .unwrap_or_else(|| Value::new(ValueKind::Null).into_handle(vm)),
                )
            }
            Some("constructor") => return value.constructor(vm),
            Some("prototype") => {
                let is_function = value.is_function();
                if is_function {
                    // Drop borrowed value because we need to re-borrow it mutably down here
                    // to set the prototype
                    drop(value);

                    let mut value = unsafe { value_cell.borrow_mut_unbounded() };
                    let func = value.as_function_mut().unwrap();
                    return func.get_or_set_prototype(&value_cell, vm);
                }
            }
            Some("length") => {
                match value.as_object().map(|o| &o.kind) {
                    Some(ObjectKind::Exotic(ExoticObject::Array(a))) => {
                        return Some(vm.create_js_value(a.elements.len() as f64).into_handle(vm))
                    }
                    Some(ObjectKind::Exotic(ExoticObject::String(s))) => {
                        return Some(vm.create_js_value(s.len() as f64).into_handle(vm))
                    }
                    _ => {}
                };
            }
            Some(key) => {
                if let Ok(idx) = key.parse::<usize>() {
                    if let Some(a) = value.as_object().and_then(Object::as_array) {
                        return a.elements.get(idx).cloned();
                    }
                }
            }
            _ => {}
        };

        if let Some(fields) = value.fields() {
            if !fields.is_empty() {
                if let Some(entry_cell) = fields.get(key) {
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
                            };
                        }
                    }
                    return Some(Handle::clone(entry_cell));
                }
            }
        }

        if let Some(proto_cell) = value.prototype(vm).as_ref() {
            Value::get_property(vm, proto_cell, key, override_this.or(Some(value_cell)))
        } else {
            None
        }
    }

    /// Adds a field
    pub fn set_property(&mut self, k: PropertyKey<'static>, v: Handle<Value>) {
        if let Some(fields) = self.fields_mut() {
            fields.insert(k, v);
        }
    }

    pub(crate) fn mark(&self) {
        if let Some(handle) = self.as_object() {
            Object::mark(handle)
        }
    }
}
