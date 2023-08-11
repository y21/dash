use dash_lexer::Lexer;
use dash_middle::compiler::CompileResult;
use dash_middle::interner::StringInterner;
use dash_middle::lexer;
use dash_middle::parser;
use dash_optimizer::type_infer::TypeInferCtx;
use dash_optimizer::OptLevel;
use dash_parser::Parser;

use crate::error::CompileError;
use crate::FunctionCompiler;

#[derive(Debug)]
pub enum CompileStrError {
    Lexer(Vec<lexer::error::Error>),
    Parser(Vec<parser::error::Error>),
    Compiler(CompileError),
}

impl<'interner> FunctionCompiler<'interner> {
    pub fn compile_str(
        interner: &'interner mut StringInterner,
        input: &str,
        opt: OptLevel,
    ) -> Result<CompileResult, CompileStrError> {
        let tokens = Lexer::new(interner, input).scan_all().map_err(CompileStrError::Lexer)?;
        let (ast, counter) = Parser::new(interner, input, tokens)
            .parse_all()
            .map_err(CompileStrError::Parser)?;

        let tcx = TypeInferCtx::new(counter);

        Self::new(opt, tcx, interner)
            .compile_ast(ast, true)
            .map_err(CompileStrError::Compiler)
    }
}
