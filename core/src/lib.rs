use std::{cell::RefCell, rc::Rc};

use compiler::compiler::Compiler;
use parser::{
    lexer::{Error as LexError, Lexer},
    parser::Parser,
    token::Error as ParseError,
};
use vm::{
    value::{
        function::{Constructor, FunctionType, UserFunction},
        object::AnyObject,
        Value,
    },
    VM,
};

use crate::vm::value::function::Receiver;

pub mod compiler;
pub mod gc;
pub mod js_std;
pub mod json;
pub mod optimizer;
pub mod parser;
#[cfg(test)]
pub mod tests;
pub mod util;
pub mod visitor;
pub mod vm;

#[derive(Debug)]
pub enum EvalError<'a> {
    LexError(Vec<LexError>),
    ParseError(Vec<ParseError<'a>>),
    VMError(vm::VMError),
}

/// Convenient function for evaluating a JavaScript source code string with default settings
/// Returns the last evaluated value
/// Consider compiling source code once and creating a new VM directly for multiple calls with same source code
pub fn eval<'a>(code: &'a str) -> Result<Option<Rc<RefCell<Value>>>, EvalError<'a>> {
    let code = code.as_ref();
    let tokens = Lexer::new(code).scan_all().map_err(EvalError::LexError)?;
    let statements = Parser::new(tokens)
        .parse_all()
        .map_err(EvalError::ParseError)?;
    let instructions = Compiler::new(statements).compile();
    let mut func = UserFunction::new(instructions, 0, FunctionType::Top, 0, Constructor::NoCtor);
    func.bind(Receiver::Bound(Value::from(AnyObject {}).into()));
    let mut vm = VM::new(func);
    let result = vm.interpret().map_err(EvalError::VMError)?;

    Ok(result)
}
