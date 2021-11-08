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
}
