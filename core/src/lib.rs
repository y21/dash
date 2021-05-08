#![feature(maybe_uninit_uninit_array, maybe_uninit_ref)]

use vm::value::UserFunction;

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

    let mut vm = vm::VM::new(UserFunction::new(instructions, 0));
    vm.interpret().map_err(EvalError::VMError)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn interpreter() {
        let code = r#"
        function F(a, b) {
            print a;
            if (a) {
                return F(a - 1, 2);
            }
        }
        F(254, 1);
        "#;

        eval(code).unwrap();
    }

    #[test]
    pub fn size() {
        dbg!(std::mem::size_of::<vm::value::Value>());
    }
}
