use core::fmt;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;

#[cfg(feature = "format")]
use serde::{Deserialize, Serialize};

use crate::interner::Symbol;
use crate::parser::expr::LiteralExpr;
use crate::parser::statement::FunctionKind;

use super::external::External;
use super::DebugSymbols;

/// The instruction buffer.
/// Uses interior mutability since we store it in a `Rc<Function>`
/// and we want to be able to optimize the bytecode
pub struct Buffer(pub Cell<Box<[u8]>>);

impl Buffer {
    pub fn with<R>(&self, fun: impl FnOnce(&[u8]) -> R) -> R {
        let buf = self.0.take();
        // this can genuinely happen for empty functions
        // (which actually shouldn't happen because we implicitly always insert a `ret` instruction),
        // but often is a bug due to calling `with` while
        // already in a `with` closure (or after unwinding), so try to save a bunch of debugging time.
        // this should _really_ only be with debug assertions, as this is very hot code
        debug_assert!(!buf.is_empty());
        let ret = fun(&buf);
        self.0.set(buf);
        ret
    }
}

#[cfg(feature = "format")]
impl Serialize for Buffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.with(|buf| buf.serialize(serializer))
    }
}

#[cfg(feature = "format")]
impl<'de> Deserialize<'de> for Buffer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Box::<[u8]>::deserialize(deserializer).map(|buf| Self(Cell::new(buf)))
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.with(|buf| buf.fmt(f))
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        self.with(|buf| Self(Cell::new(Box::from(buf))))
    }
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Function {
    pub name: Option<Symbol>,
    pub buffer: Buffer,
    pub ty: FunctionKind,
    pub locals: usize,
    pub params: usize,
    pub constants: Box<[Constant]>,
    pub externals: Box<[External]>,
    pub r#async: bool,
    /// If the parameter list uses the rest operator ..., then this will be Some(local_id)
    pub rest_local: Option<u16>,
    // JIT-poisoned code regions (instruction pointers)
    // TODO: refactor this a bit so this isn't "visible" to e.g. the bytecode compiler with builder pattern
    pub poison_ips: RefCell<HashSet<usize>>,
    pub source: Rc<str>,
    pub debug_symbols: DebugSymbols,
    pub references_arguments: bool,
}

impl Function {
    pub fn poison_ip(&self, ip: usize) {
        self.poison_ips.borrow_mut().insert(ip);
    }

    pub fn is_poisoned_ip(&self, ip: usize) -> bool {
        self.poison_ips.borrow().contains(&ip)
    }
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    String(Symbol),
    Identifier(Symbol),
    Boolean(bool),
    Function(Rc<Function>),
    // Boxed because this otherwise bloats the enum way too much.
    // This makes evaluating regex constants slower but they're *far* less common than e.g. number literals
    // TODO: avoid cloning `Constant`s when turning them into Values,
    // because there's no point in cloning this box
    Regex(Box<(dash_regex::ParsedRegex, dash_regex::Flags, Symbol)>),
    Null,
    Undefined,
}

impl Constant {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Constant::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<Symbol> {
        match self {
            Constant::String(s) => Some(*s),
            _ => None,
        }
    }

    pub fn as_identifier(&self) -> Option<Symbol> {
        match self {
            Constant::Identifier(s) => Some(*s),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Constant::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn from_literal(expr: &LiteralExpr) -> Self {
        match expr {
            LiteralExpr::Number(n) => Self::Number(*n),
            LiteralExpr::Identifier(s) => Self::Identifier(*s),
            LiteralExpr::String(s) => Self::String(*s),
            LiteralExpr::Boolean(b) => Self::Boolean(*b),
            LiteralExpr::Null => Self::Null,
            LiteralExpr::Undefined => Self::Undefined,
            LiteralExpr::Regex(regex, flags, source) => Self::Regex(Box::new((regex.clone(), *flags, *source))),
        }
    }
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct ConstantPool {
    constants: Vec<Constant>,
}

pub struct LimitExceededError;
impl ConstantPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, constant: Constant) -> Result<u16, LimitExceededError> {
        if self.constants.len() > u16::MAX as usize {
            Err(LimitExceededError)
        } else {
            let id = self.constants.len() as u16;
            self.constants.push(constant);
            Ok(id)
        }
    }

    pub fn into_vec(self) -> Vec<Constant> {
        self.constants
    }
}

impl Deref for ConstantPool {
    type Target = [Constant];

    fn deref(&self) -> &Self::Target {
        &self.constants
    }
}
