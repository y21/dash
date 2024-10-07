use core::fmt;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use dash_regex::{Flags, ParsedRegex};

use crate::index_type;
use crate::indexvec::IndexThinVec;
use crate::interner::Symbol;
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
impl serde::Serialize for Buffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.with(|buf| buf.serialize(serializer))
    }
}

#[cfg(feature = "format")]
impl<'de> serde::Deserialize<'de> for Buffer {
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

#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Function {
    pub name: Option<Symbol>,
    pub buffer: Buffer,
    pub ty: FunctionKind,
    pub locals: usize,
    pub params: usize,
    pub constants: ConstantPool,
    pub externals: Box<[External]>,
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
index_type!(NumberConstant u16);
index_type!(BooleanConstant u16);
index_type!(FunctionConstant u16);
index_type!(RegexConstant u16);
index_type!(SymbolConstant u16);

#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug, Clone)]
pub struct ConstantPool {
    pub numbers: IndexThinVec<f64, NumberConstant>,
    /// Strings and identifiers
    pub symbols: IndexThinVec<Symbol, SymbolConstant>,
    pub booleans: IndexThinVec<bool, BooleanConstant>,
    pub functions: IndexThinVec<Rc<Function>, FunctionConstant>,
    pub regexes: IndexThinVec<(ParsedRegex, Flags, Symbol), RegexConstant>,
}

pub struct LimitExceededError;

macro_rules! define_push_methods {
    ($($method:ident($field:ident, $valty:ty) -> $constant:ty),*) => {
        $(
            pub fn $method(
                &mut self,
                val: $valty,
            ) -> Result<$constant, LimitExceededError> {
                self.$field.try_push(val).map_err(|_| LimitExceededError)
            }
        )*
    };
}

impl ConstantPool {
    define_push_methods!(
        add_number(numbers, f64) -> NumberConstant,
        add_symbol(symbols, Symbol) -> SymbolConstant,
        add_boolean(booleans, bool) -> BooleanConstant,
        add_function(functions, Rc<Function>) -> FunctionConstant,
        add_regex(regexes, (ParsedRegex, Flags, Symbol)) -> RegexConstant
    );
}
