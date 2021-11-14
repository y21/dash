use std::collections::HashMap;
use std::fmt::Debug;

use crate::gc::Handle;
use crate::vm::VM;

use super::exotic::Exotic;
use super::generator::GeneratorIterator;
use super::promise::Promise;
use super::symbol::Symbol;
use super::weak::Weak;
use super::{array::Array, function::FunctionKind};
use super::{PropertyKey, Value};

/// A JavaScript exotic object
///
/// Any kind of object that is "magic" in some way is exotic.
/// For example, functions are callable objects.
#[derive(Debug, Clone)]
pub enum ExoticObject {
    /// A JavaScript String
    String(String),
    /// A JavaScript function
    Function(FunctionKind),
    /// A JavaScript array
    Array(Array),
    /// A JavaScript weak type
    Weak(Weak),
    /// A JavaScript promise
    Promise(Promise),
    /// A JavaScript iterator over a generator function
    GeneratorIterator(GeneratorIterator),
    /// A JavaScript symbol
    Symbol(Symbol),
    /// Custom exotic types
    Custom(Box<dyn Exotic>),
}

/// A JavaScript object type
#[derive(Debug, Clone)]
pub enum ObjectKind {
    /// Exotic object
    Exotic(ExoticObject),
    /// Ordinary, regular object
    Ordinary,
}

/// A JavaScript object
#[derive(Debug, Clone)]
pub struct Object {
    /// The object's type
    pub kind: ObjectKind,
    /// The fields of this value
    pub fields: HashMap<PropertyKey<'static>, Value>,
    /// This value's constructor
    pub constructor: Option<Handle<Object>>,
    /// This value's [[Prototype]]
    pub prototype: Option<Handle<Object>>,
}

impl Object {
    /// Creates a new object with no prototype and constructor set
    pub fn new(kind: ObjectKind) -> Self {
        Self {
            kind,
            fields: HashMap::new(),
            constructor: None,
            prototype: None,
        }
    }

    /// Registers this object for garbage collection and returns a handle to it
    // TODO: re-think whether this is fine to not be unsafe?
    pub fn into_handle(self, vm: &VM) -> Handle<Self> {
        vm.gc.borrow_mut().register(self)
    }

    pub(crate) fn mark(this: &Handle<Object>) {
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

        if let Some(proto) = &this.prototype {
            Self::mark(&proto)
        }

        if let Some(constructor) = &this.constructor {
            Self::mark(&constructor)
        }

        for (key, value) in this.fields.iter() {
            key.mark();
            value.mark();
        }

        match &this.kind {
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
        };
    }

    /// Updates the internal properties ([[Prototype]] and constructor)
    /// of this JavaScript value
    pub fn update_internal_properties(&mut self, proto: &Handle<Object>, ctor: &Handle<Object>) {
        self.prototype = Some(Handle::clone(proto));
        self.constructor = Some(Handle::clone(ctor));
    }

    /// Tries to detect the [[Prototype]] and constructor of this object, and updates it
    pub fn detect_internal_properties(&mut self, vm: &VM) {
        let statics = &vm.statics;

        match &self.kind {
            ObjectKind::Exotic(ExoticObject::Promise(_)) => {
                self.update_internal_properties(&statics.promise_proto, &statics.promise_ctor)
            }
            ObjectKind::Exotic(ExoticObject::String(_)) => {
                self.update_internal_properties(&statics.string_proto, &statics.string_ctor)
            }
            ObjectKind::Exotic(ExoticObject::Function(_)) => {
                self.update_internal_properties(&statics.function_proto, &statics.function_ctor)
            }
            ObjectKind::Exotic(ExoticObject::Array(_)) => {
                self.update_internal_properties(&statics.array_proto, &statics.array_ctor)
            }
            ObjectKind::Exotic(ExoticObject::GeneratorIterator(_)) => self
                .update_internal_properties(
                    &statics.generator_iterator_proto,
                    &statics.object_ctor, // TODO: generator iterator ctor
                ),
            ObjectKind::Exotic(ExoticObject::Symbol(_)) => {
                self.update_internal_properties(&statics.symbol_proto, &statics.symbol_ctor)
            }
            ObjectKind::Ordinary | ObjectKind::Exotic(ExoticObject::Custom(_)) => {
                self.update_internal_properties(&statics.object_proto, &statics.object_ctor)
            }
            ObjectKind::Exotic(ExoticObject::Weak(Weak::Set(_))) => {
                self.update_internal_properties(&statics.weakset_proto, &statics.weakset_ctor)
            }
            ObjectKind::Exotic(ExoticObject::Weak(Weak::Map(_))) => {
                self.update_internal_properties(&statics.weakmap_proto, &statics.weakmap_ctor)
            }
            _ => {}
        }
    }

    /// Returns whether this value is a primitive
    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, ObjectKind::Exotic(ExoticObject::String(_)))
    }

    /// Returns whether this value is callable
    pub fn is_callable(&self) -> bool {
        matches!(self.kind, ObjectKind::Exotic(ExoticObject::Function(_)))
    }

    /// Checks whether this value (or one of the values in its prototype chain) contains a field
    pub fn has_property(&self, vm: &VM, key: PropertyKey<'_>) -> bool {
        if self.fields.contains_key(&key) {
            return true;
        }

        self.prototype
            .as_ref()
            .map(|x| x.borrow(vm).has_property(vm, key))
            .unwrap_or(false)
    }

    /// Sets a property on this object
    pub fn set_property<K, V>(&self, key: K, value: V)
    where
        K: Into<PropertyKey<'static>>,
        V: Into<Value>,
    {
        self.fields.insert(key.into(), value.into());
    }

    /// Looks up a property and goes through exotic property matching
    ///
    /// For a direct field lookup, use [Value::get_field]
    pub fn get_property(&self, vm: &VM, key: PropertyKey<'_>) -> Option<Value> {
        if key.is_constructor() {
            return self.constructor.clone().map(Into::into);
        }

        if key.is_prototype() {
            return self.prototype.clone().map(Into::into);
        }

        if key.is_function_prototype() && self.is_callable() {
            if let Some((func, prototype)) = self.as_function().zip(self.prototype.as_ref()) {
                return func.get_or_set_prototype(prototype, vm).map(Into::into);
            }
        }

        if let Some(value) = self.fields.get(&key) {
            return Some(value.clone());
        }

        self.prototype
            .as_ref()
            .and_then(|x| x.borrow(vm).get_property(vm, key))
    }
}
