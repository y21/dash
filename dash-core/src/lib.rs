//! JavaScript implementation written in Rust

// #![deny(missing_docs)]
#![allow(unused_unsafe, dead_code, unused_variables)]

use std::borrow::Cow;

use compiler::error::CompileError;
use parser::{consteval::OptLevel, lexer::Error as LexError, token::Error as ParseError};
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
            Self::CompileError(c) => Cow::Owned(format!("{:?}", c)),
            Self::VmError(v) => Cow::Owned(format!("{:?}", v)),
        }
    }
}

pub fn eval(input: &str) -> Result<(Vm, Value), EvalError> {
    let mut vm = Vm::new();
    let tokens = Parser::from_str(input).map_err(|e| EvalError::LexError(e))?;
    let ast = tokens
        .parse_all(OptLevel::Aggressive)
        .map_err(|e| EvalError::ParseError(e))?;
    let compiled = FunctionCompiler::compile_ast(ast).map_err(|e| EvalError::CompileError(e))?;
    let frame = Frame {
        local_count: compiled.locals,
        buffer: compiled.instructions.into_boxed_slice(),
        constants: compiled.cp.into_vec().into_boxed_slice(),
        ip: 0,
    };
    let val = vm.execute_frame(frame).map_err(|e| EvalError::VmError(e))?;
    Ok((vm, val))
}
