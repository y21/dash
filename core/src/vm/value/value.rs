use core::fmt::Debug;
use std::{
    borrow::Cow,
    cell::RefCell,
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
    Symbol(Handle<Value>),
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
    pub fn inspect(&self, depth: u32) -> String {
        match self {
            PropertyKey::String(s) => s.to_string(),
            PropertyKey::Symbol(s) => {
                let s = unsafe { s.borrow_unbounded() };
                s.inspect(depth).to_string()
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

impl From<Handle<Value>> for PropertyKey<'_> {
    fn from(h: Handle<Value>) -> Self {
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
            vm.stack
                .push(Value::new(ValueKind::Undefined).into_handle(vm));
        }

        match vm.execute_frame(frame, false) {
            Ok(DispatchResult::Return(Some(r)) | DispatchResult::Yield(Some(r))) => Ok(r),
            Ok(_) => Ok(Value::new(ValueKind::Undefined).into_handle(vm)),
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
        if let ValueKind::Object(obj) = &mut self.kind {
            obj.prototype = Some(Handle::clone(proto));
            obj.constructor = Some(Handle::clone(ctor));
        }
    }

    /// Updates the [[Prototype]] of this JavaScript value
    pub fn set_prototype(&mut self, proto: Option<&Handle<Value>>) {
        if let ValueKind::Object(obj) = &mut self.kind {
            obj.prototype = proto.cloned();
        }
    }

    /// Tries to detect the [[Prototype]] and constructor of this value given self.kind, and updates it
    pub fn detect_internal_properties(&mut self, vm: &VM) {
        let statics = &vm.statics;

        match self.as_object().map(|x| &x.kind) {
            Some(ObjectKind::Exotic(ExoticObject::Promise(_))) => {
                self.update_internal_properties(&statics.promise_proto, &statics.promise_ctor)
            }
            Some(ObjectKind::Exotic(ExoticObject::String(_))) => {
                self.update_internal_properties(&statics.string_proto, &statics.string_ctor)
            }
            Some(ObjectKind::Exotic(ExoticObject::Function(_))) => {
                self.update_internal_properties(&statics.function_proto, &statics.function_ctor)
            }
            Some(ObjectKind::Exotic(ExoticObject::Array(_))) => {
                self.update_internal_properties(&statics.array_proto, &statics.array_ctor)
            }
            Some(ObjectKind::Exotic(ExoticObject::GeneratorIterator(_))) => self
                .update_internal_properties(
                    &statics.generator_iterator_proto,
                    &statics.object_ctor, // TODO: generator iterator ctor
                ),
            Some(ObjectKind::Exotic(ExoticObject::Symbol(_))) => {
                self.update_internal_properties(&statics.symbol_proto, &statics.symbol_ctor)
            }
            Some(ObjectKind::Ordinary | ObjectKind::Exotic(ExoticObject::Custom(_))) => {
                self.update_internal_properties(&statics.object_proto, &statics.object_ctor)
            }
            Some(ObjectKind::Exotic(ExoticObject::Weak(JsWeak::Set(_)))) => {
                self.update_internal_properties(&statics.weakset_proto, &statics.weakset_ctor)
            }
            Some(ObjectKind::Exotic(ExoticObject::Weak(JsWeak::Map(_)))) => {
                self.update_internal_properties(&statics.weakmap_proto, &statics.weakmap_ctor)
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
            ValueKind::Object(o) => matches!(&o.kind, ObjectKind::Exotic(ExoticObject::String(_))),
        }
    }

    /// Returns whether this value is callable
    pub fn is_callable(&self) -> bool {
        match &self.kind {
            ValueKind::Object(o) => {
                matches!(&o.kind, ObjectKind::Exotic(ExoticObject::Function(_)))
            }
            _ => false,
        }
    }

    /// Checks whether this value is strictly a function
    pub fn is_function(&self) -> bool {
        self._typeof() == Typeof::Function
    }

    /// Returns a reference to the [[Prototype]] of this value, if it has one
    pub fn prototype(&self, vm: &VM) -> Option<Handle<Value>> {
        match &self.kind {
            ValueKind::Bool(_) => Some(Handle::clone(&vm.statics.boolean_proto)),
            ValueKind::Number(_) => Some(Handle::clone(&vm.statics.number_proto)),
            ValueKind::Null | ValueKind::Undefined => None,
            ValueKind::Object(o) => o.prototype.as_ref().cloned(),
            _ => None,
        }
    }

    /// Returns a reference to the inner [[Prototype]] of this value if it is an object
    ///
    /// The prototype of primitive values never changes
    pub fn object_prototype(&self) -> Option<Handle<Value>> {
        self.as_object().and_then(|o| o.prototype.as_ref().cloned())
    }

    /// Returns a reference to the constructor of this value, if it has one
    pub fn constructor(&self, vm: &VM) -> Option<Handle<Value>> {
        match &self.kind {
            ValueKind::Bool(_) => Some(Handle::clone(&vm.statics.boolean_ctor)),
            ValueKind::Number(_) => Some(Handle::clone(&vm.statics.number_ctor)),
            ValueKind::Null | ValueKind::Undefined => None,
            ValueKind::Object(o) => o.constructor.as_ref().cloned(),
            _ => None,
        }
    }

    /// Returns a reference to the inner constructor of this value if it is an object
    ///
    /// The constructor of primitive values never changes
    pub fn object_constructor(&self) -> Option<Handle<Value>> {
        self.as_object()
            .and_then(|o| o.constructor.as_ref().cloned())
    }

    /// Unwraps o, or returns undefined if it is None
    pub fn unwrap_or_undefined(o: Option<Handle<Self>>, vm: &VM) -> Handle<Self> {
        o.unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(vm))
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

        if let Some(proto) = this.object_prototype() {
            Value::mark(&proto)
        }

        if let Some(constructor) = this.object_constructor() {
            Value::mark(&constructor)
        }

        if let Some(fields) = this.fields() {
            for (key, value) in fields.iter() {
                key.mark();
                Value::mark(value)
            }
        }

        match &this.kind {
            ValueKind::Object(o) => match &o.kind {
                ObjectKind::Exotic(ExoticObject::Array(a)) => {
                    for handle in &a.elements {
                        Value::mark(handle)
                    }
                }
                ObjectKind::Exotic(ExoticObject::GeneratorIterator(gen)) => gen.mark(),
                ObjectKind::Exotic(ExoticObject::Function(f)) => f.mark(),
                ObjectKind::Exotic(ExoticObject::Promise(_)) => todo!(),
                ObjectKind::Exotic(ExoticObject::Custom(_)) => {
                    panic!("Custom GC marking is unsupported")
                }
                ObjectKind::Exotic(ExoticObject::Weak(_)) => todo!(), // weak objects don't exist yet
                // Other object types that do not contain handles that need to be marked
                ObjectKind::Exotic(ExoticObject::String(_)) => {}
                ObjectKind::Exotic(ExoticObject::Symbol(_)) => {}
                ObjectKind::Ordinary => {}
            },
            _ => {}
        };
    }
}
