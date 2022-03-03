//! JavaScript implementation written in Rust

// #![deny(missing_docs)]
#![allow(unused_unsafe, dead_code, unused_variables)]

use core::fmt;
use std::error::Error;

use compiler::error::CompileError;
use optimizer::consteval::OptLevel;
use parser::{lexer::Error as LexError, token::Error as ParseError};
use vm::{value::Value, Vm};

use crate::{compiler::FunctionCompiler, parser::parser::Parser, vm::frame::Frame};

/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// AST to bytecode compiler
pub mod compiler;
/// Garbage collector, to free resources that are no longer used
pub mod gc;
/// JavaScript standard library
pub mod js_std;
pub mod optimizer;
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
    /// A compilation error
    CompileError(CompileError),
    /// A VM execution error
    VmError(Value),
}

impl<'a> fmt::Display for EvalError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompileError(c) => write!(f, "{:?}", c),
            Self::LexError(l) => {
                for e in l {
                    write!(f, "{}\n", e.to_string())?;
                }
                Ok(())
            }
            Self::ParseError(p) => {
                for e in p {
                    write!(f, "{}\n", e.to_string())?;
                }
                Ok(())
            }
            Self::VmError(v) => write!(f, "{:?}", v),
        }
    }
}

impl<'a> Error for EvalError<'a> {}

pub fn eval(input: &str) -> Result<(Vm, Value), EvalError> {
    let mut vm = Vm::new();
    let tokens = Parser::from_str(input).map_err(EvalError::LexError)?;
    let mut ast = tokens.parse_all().map_err(EvalError::ParseError)?;
    optimizer::optimize_ast(&mut ast, OptLevel::Aggressive);
    let compiled = FunctionCompiler::new()
        .compile_ast(ast)
        .map_err(EvalError::CompileError)?;
    let frame = Frame::from(compiled);
    let val = vm.execute_frame(frame).map_err(|e| EvalError::VmError(e))?;
    Ok((vm, val))
}
