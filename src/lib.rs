#![feature(maybe_uninit_uninit_array, maybe_uninit_ref)]

use vm::value::Value;

pub mod compiler;
pub mod gc;
pub mod optimizer;
pub mod parser;
pub mod util;
pub mod visitor;
pub mod vm;

#[derive(Debug)]
pub enum EvalError {
    VMError(vm::VMError),
}

/// Convenient function for evaluating a JavaScript source code string
/// Returns the last evaluated value
pub fn eval(code: impl AsRef<str>) -> Result<Option<Value>, EvalError> {
    let code = code.as_ref();
    let tokens = parser::lexer::Lexer::new(code).scan_all();
    let statements = parser::parser::Parser::new(tokens).parse_all();
    let instructions = compiler::compiler::Compiler::new(statements).compile();
    let mut vm = vm::VM::new(instructions);
    vm.interpret().map_err(EvalError::VMError)?;

    Ok(Value::try_into_inner(vm.stack.pop()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn interpreter() {
        dbg!(eval("if(true) 1+2"));
    }
}
