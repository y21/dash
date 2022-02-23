//! JavaScript implementation written in Rust

// #![deny(missing_docs)]
#![allow(unused_unsafe, dead_code, unused_variables)]

use std::borrow::Cow;

// use agent::Agent;
// use compiler::compiler::CompileError;
// use gc::Handle;
use parser::{lexer::Error as LexError, token::Error as ParseError};
// use vm::{value::Value, FromStrError, VMError, VM};

/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// AST to bytecode compiler
pub mod compiler;
/// Garbage collector, to free resources that are no longer used
pub mod gc;
/// JavaScript standard library
pub mod js_std;
/// JavaScript lexer and parser
pub mod parser;
/// Utility types and functions used in this implementation
pub mod util;
/// Bytecode VM
pub mod vm;

/// An error that occurred during a call to eval
#[derive(Debug)]
pub enum EvalError<'a> {
    /// A lexer error
    LexError(Vec<LexError<'a>>),
    /// A parser error
    ParseError(Vec<ParseError<'a>>),
}

impl<'a> EvalError<'a> {
    /// Formats this error by calling to_string on the underlying error
    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::LexError(l) => Cow::Owned(
                l.iter()
                    .map(|e| e.to_string().to_string())
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
            Self::ParseError(p) => Cow::Owned(
                p.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
        }
    }
}
