use core::fmt;
use std::cell::Cell;
use std::rc::Rc;

use dash_regex::Regex;

use crate::compiler::external::ExternalId;
use crate::compiler::scope::BackLocalId;
use crate::index_type;
use crate::indexthinvec::IndexThinVec;
use crate::indexvec::IndexVec;
use crate::interner::Symbol;
use crate::parser::statement::FunctionKind;

use super::DebugSymbols;
use super::external::External;

/// The instruction buffer.
/// Uses interior mutability since we store it in a `Rc<Function>`
/// and we want to be able to optimize the bytecode
pub struct Buffer(Cell<Box<[u8]>>);

impl Buffer {
    pub fn new(buf: Box<[u8]>) -> Self {
        Self(Cell::new(buf))
    }

    #[inline]
    pub fn at(&self, ip: u32) -> u8 {
        // SAFETY: while we're holding the `&[u8]`, there cannot exist other mutable references to that buffer
        let slice = unsafe { &*(*self.0.as_ptr()) };
        slice[ip as usize]
    }

    #[inline]
    pub fn copy_range<const N: usize>(&self, ip: u32) -> [u8; N] {
        // SAFETY: while we're holding the `&[u8]`, there cannot exist other mutable references to that buffer
        let slice = unsafe { &*(*self.0.as_ptr()) };
        slice[ip as usize..ip as usize + N]
            .try_into()
            .expect("Failed to copy range")
    }

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
    pub locals: u16,
    pub params: u16,
    pub constants: ConstantPool,
    pub externals: IndexVec<External, ExternalId>,
    /// If the parameter list uses the rest operator ..., then this will be Some(local_id)
    pub rest_local: Option<BackLocalId>,
    pub source: Rc<str>,
    pub debug_symbols: DebugSymbols,
    pub references_arguments: bool,
    pub has_extends_clause: bool,
}

index_type!(
    #[derive(Copy, Default, Debug, Clone)]
    pub struct NumberConstant(pub u16);
    #[derive(Copy, Default, Debug, Clone)]
    pub struct BooleanConstant(pub u16);
    #[derive(Copy, Default, Debug, Clone)]
    pub struct FunctionConstant(pub u16);
    #[derive(Copy, Default, Debug, Clone)]
    pub struct RegexConstant(pub u16);
    #[derive(Copy, Default, Debug, Clone)]
    pub struct SymbolConstant(pub u16);
);

#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug, Clone)]
pub struct ConstantPool {
    pub numbers: IndexThinVec<f64, NumberConstant>,
    /// Strings and identifiers
    pub symbols: IndexThinVec<Symbol, SymbolConstant>,
    pub booleans: IndexThinVec<bool, BooleanConstant>,
    pub functions: IndexThinVec<Rc<Function>, FunctionConstant>,
    pub regexes: IndexThinVec<(Regex, Symbol), RegexConstant>,
}

pub struct LimitExceededError;

macro_rules! define_push_methods {
    ($($method:ident($field:ident, $valty:ty) -> $constant:ty),*) => {
        $(
            pub fn $method(
                &mut self,
                val: $valty,
            ) -> Result<$constant, LimitExceededError> {
                match self.$field.try_push(val) {
                    Some(index) => Ok(index),
                    None => Err(LimitExceededError),
                }
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
        add_regex(regexes, (Regex, Symbol)) -> RegexConstant
    );

    pub fn shrink_to_fit(&mut self) {
        let Self {
            numbers,
            symbols,
            booleans,
            functions,
            regexes,
        } = self;
        numbers.shrink_to_fit();
        symbols.shrink_to_fit();
        booleans.shrink_to_fit();
        functions.shrink_to_fit();
        regexes.shrink_to_fit();
    }
}
