//! JavaScript implementation written in Rust

#![deny(missing_docs)]
#![allow(unused_unsafe)]

use std::{borrow::Cow, cell::RefCell, rc::Rc};

use agent::Agent;
use compiler::compiler::CompileError;
use parser::{lexer::Error as LexError, token::Error as ParseError};
use vm::{value::Value, FromStrError, VMError, VM};

/// Allows embedders to control behavior
pub mod agent;
/// AST to bytecode compiler
pub mod compiler;
/// Garbage collector, to free resources that are no longer used
pub mod gc;
/// JavaScript standard library
pub mod js_std;
/// JSON parser and serializer
pub mod json;
/// Applies optimizations to JavaScript code at compile time
pub mod optimizer;
/// JavaScript lexer and parser
pub mod parser;
#[cfg(test)]
pub mod tests;
/// Utility types and functions used in this implementation
pub mod util;
/// Visitor trait, used to walk the AST
pub mod visitor;
/// Bytecode VM
pub mod vm;

/// An error that occurred during a call to [eval]
#[derive(Debug)]
pub enum EvalError<'a> {
    /// A lexer error
    LexError(Vec<LexError<'a>>),
    /// A parser error
    ParseError(Vec<ParseError<'a>>),
    /// A compiler error
    CompileError(CompileError<'a>),
    /// A VM (runtime) error
    VMError(vm::VMError),
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
            Self::CompileError(c) => c.to_string(),
            Self::VMError(e) => e.to_string(),
        }
    }
}

impl<'a> From<FromStrError<'a>> for EvalError<'a> {
    fn from(value: FromStrError<'a>) -> Self {
        match value {
            FromStrError::LexError(l) => Self::LexError(l),
            FromStrError::ParseError(p) => Self::ParseError(p),
            FromStrError::CompileError(c) => Self::CompileError(c),
        }
    }
}

impl<'a> From<VMError> for EvalError<'a> {
    fn from(value: VMError) -> Self {
        Self::VMError(value)
    }
}

/// Convenient function for evaluating a JavaScript source code string with default settings.
/// Returns the last value. Async tasks are not evaluated.
pub fn eval<'a, A: Agent + 'static>(
    code: &'a str,
    agent: Option<A>,
) -> Result<Option<Rc<RefCell<Value>>>, EvalError<'a>> {
    let mut vm = VM::from_str(code, agent)?;
    let result = vm.interpret()?;

    Ok(result)
}
