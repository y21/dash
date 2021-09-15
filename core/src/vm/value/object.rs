use std::fmt::Debug;

use super::exotic::Exotic;
use super::generator::GeneratorIterator;
use super::promise::Promise;
use super::symbol::Symbol;
use super::weak::Weak;
use super::{array::Array, function::FunctionKind};

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

/// A JavaScript object
#[derive(Debug, Clone)]
pub enum Object {
    /// Exotic object
    Exotic(ExoticObject),
    /// Ordinary, regular object
    Ordinary,
}
