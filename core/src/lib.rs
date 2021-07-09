//! JavaScript implementation written in Rust

// #![deny(missing_docs)]

use std::{borrow::Cow, cell::RefCell, rc::Rc};

use agent::Agent;
use compiler::compiler::{CompileError, Compiler, FunctionKind};
use parser::{
    lexer::{Error as LexError, Lexer},
    parser::Parser,
    token::Error as ParseError,
};
use util::MaybeOwned;
use vm::{
    value::{
        function::{Constructor, FunctionType, UserFunction},
        object::AnyObject,
        Value,
    },
    VM,
};

use crate::vm::value::function::Receiver;

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

/// Convenient function for evaluating a JavaScript source code string with default settings.
/// Returns the last value. Async tasks are not evaluated.
pub fn eval<'a, A: Agent>(
    code: &'a str,
    agent: Option<A>,
) -> Result<Option<Rc<RefCell<Value>>>, EvalError<'a>> {
    let tokens = Lexer::new(code).scan_all().map_err(EvalError::LexError)?;
    let statements = Parser::new(code, tokens)
        .parse_all()
        .map_err(EvalError::ParseError)?;
    let instructions = Compiler::new(
        statements,
        agent.map(MaybeOwned::Owned),
        FunctionKind::Function,
    )
    .compile()
    .map_err(EvalError::CompileError)?;
    let mut func = UserFunction::new(instructions, 0, FunctionType::Top, 0, Constructor::NoCtor);
    func.bind(Receiver::Bound(Value::from(AnyObject {}).into()));
    let mut vm = VM::new(func);
    let result = vm.interpret().map_err(EvalError::VMError)?;

    Ok(result)
}
