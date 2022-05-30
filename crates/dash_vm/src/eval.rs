use std::fmt;

use dash_compiler::error::CompileError;
use dash_compiler::FunctionCompiler;
use dash_lexer::Lexer;
use dash_middle::lexer;
use dash_middle::parser;
use dash_middle::util;
use dash_optimizer::consteval::OptLevel;
use dash_parser::Parser;

use crate::frame::Frame;
use crate::value::Value;
use crate::Vm;

#[derive(Debug)]
pub enum EvalError<'a> {
    Lexer(Vec<lexer::error::Error<'a>>),
    Parser(Vec<parser::error::Error<'a>>),
    Compiler(CompileError),
    Exception(Value),
}

impl<'a> fmt::Display for EvalError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::Lexer(errors) => {
                util::fmt_group(f, &errors, "\n\n")?;
            }
            EvalError::Parser(errors) => {
                util::fmt_group(f, &errors, "\n\n")?;
            }
            EvalError::Compiler(error) => {
                writeln!(f, "{:?}", error)?;
            }
            EvalError::Exception(value) => {
                writeln!(f, "Exception: {:?}", value)?;
            }
        }

        Ok(())
    }
}

impl Vm {
    pub fn eval<'a>(&mut self, input: &'a str, opt: OptLevel) -> Result<Value, EvalError<'a>> {
        let tokens = Lexer::new(input).scan_all().map_err(EvalError::Lexer)?;
        let mut ast = Parser::new(input, tokens).parse_all().map_err(EvalError::Parser)?;
        dash_optimizer::optimize_ast(&mut ast, opt);

        let cr = FunctionCompiler::new()
            .compile_ast(ast, true)
            .map_err(EvalError::Compiler)?;
        let frame = Frame::from_compile_result(cr);
        let val = self.execute_frame(frame).map_err(EvalError::Exception)?;
        Ok(val.into_value())
    }
}
