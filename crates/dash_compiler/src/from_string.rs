use dash_lexer::Lexer;
use dash_middle::compiler::CompileResult;
use dash_middle::lexer;
use dash_middle::parser;
use dash_optimizer::type_infer::TypeInferCtx;
use dash_optimizer::OptLevel;
use dash_parser::Parser;

use crate::error::CompileError;
use crate::FunctionCompiler;

#[derive(Debug)]
pub enum CompileStrError<'a> {
    Lexer(Vec<lexer::error::Error<'a>>),
    Parser(Vec<parser::error::Error<'a>>),
    Compiler(CompileError),
}

impl<'a> FunctionCompiler<'a> {
    pub fn compile_str(input: &'a str, opt: OptLevel) -> Result<CompileResult, CompileStrError<'a>> {
        let tokens = Lexer::new(input).scan_all().map_err(CompileStrError::Lexer)?;
        let (ast, counter) = Parser::new(input, tokens)
            .parse_all()
            .map_err(CompileStrError::Parser)?;

        let tcx = TypeInferCtx::new(counter);

        Self::new(opt, tcx)
            .compile_ast(ast, true)
            .map_err(CompileStrError::Compiler)
    }
}
