use std::{cell::RefCell, rc::Rc};

use compiler::compiler::Compiler;
use parser::{lexer::Lexer, parser::Parser};
use vm::{
    value::{
        function::{FunctionType, NativeFunction, UserFunction},
        object::{AnyObject, Object},
        Value, ValueKind,
    },
    VM,
};

use crate::vm::value::function::Receiver;

pub mod compiler;
pub mod gc;
pub mod js_std;
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
pub fn eval(code: impl AsRef<str>) -> Result<Option<Rc<RefCell<Value>>>, EvalError> {
    let code = code.as_ref();
    let tokens = Lexer::new(code).scan_all().unwrap();
    let statements = Parser::new(tokens).parse_all().unwrap();
    let instructions = Compiler::new(statements).compile();
    let mut func = UserFunction::new(instructions, 0, FunctionType::Top, 0, false);
    func.bind(Receiver::Bound(Value::from(AnyObject {}).into()));
    let mut vm = VM::new(func);
    let result = vm.interpret().map_err(EvalError::VMError)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::vm::stack::Stack;

    use super::*;

    #[test]
    pub fn recursion() {
        let result = eval(
            r#"function F(n) {
            function __nothing__() {}

            if (n) {
                let f = n * 2;
                console.log(f);
                return F(n - 1);
            }
        }

        F(16);"#,
        )
        .unwrap();
        println!("{:?}", result);
    }

    #[test]
    pub fn inner_fn() {
        eval(
            r#"
                function a(b, c) {
                    function d(e, f) {
                        return e * f * c;
                    }

                    return d(b, c);
                }

                print a(3,3);
        "#,
        )
        .unwrap();
    }

    #[test]
    pub fn upvalues() {
        eval(
            r#"
            function a(b) {
                function c(d) {
                    print b * d;
                }
                return c;
            }
            a(2)(4);
        "#,
        )
        .unwrap();
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
