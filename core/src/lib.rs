#![feature(maybe_uninit_uninit_array, maybe_uninit_ref)]

use vm::value::{FunctionType, UserFunction};

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
pub fn eval(code: impl AsRef<str>) -> Result<(), EvalError> {
    let code = code.as_ref();
    let tokens = parser::lexer::Lexer::new(code).scan_all();
    let statements = parser::parser::Parser::new(tokens).parse_all();
    let instructions = compiler::compiler::Compiler::new(statements).compile();
    let mut vm = vm::VM::new(UserFunction::new(instructions, 0, FunctionType::Top));
    vm.interpret().map_err(EvalError::VMError)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::vm::stack::Stack;

    use super::*;

    #[test]
    pub fn interpreter() {
        let code = r#"
        function F(n) {
            if (n) {
                let f = n * 2;
                print f;
                return F(n - 1);
            }
        }

        F(16);
        "#;

        eval(code).unwrap();
    }

    #[test]
    pub fn size() {
        dbg!(std::mem::size_of::<vm::value::Value>());
    }

    #[test]
    pub fn stack_memory_leak() {
        let mut s = Stack::<_, 5>::new();
        s.push(String::from("test"));
    }
}
