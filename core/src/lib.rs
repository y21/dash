#![feature(maybe_uninit_uninit_array, maybe_uninit_ref)]

use std::{cell::RefCell, rc::Rc};

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
pub fn eval(code: impl AsRef<str>) -> Result<Rc<RefCell<Value>>, EvalError> {
    let code = code.as_ref();
    let tokens = parser::lexer::Lexer::new(code).scan_all();
    let statements = parser::parser::Parser::new(tokens).parse_all();
    let instructions = compiler::compiler::Compiler::new(statements).compile();
    let mut vm = vm::VM::new(instructions);
    vm.interpret().map_err(EvalError::VMError)?;

    Ok(vm.stack.pop())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn interpreter() {
        /*let code = r#"
            function hello() {
                let a = 1 + 2 * 3;
                let b = a * 3;
                b
            }
        "#;*/
        let code = r#"if (true) 1+2;"#;

        let res = Value::try_into_inner(eval(code).unwrap()).unwrap();
        dbg!(res);
    }

    #[test]
    pub fn size() {
        dbg!(std::mem::size_of::<vm::value::Value>());
    }
}
