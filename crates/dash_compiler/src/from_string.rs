use dash_lexer::Lexer;
use dash_middle::compiler::CompileResult;
use dash_middle::interner::StringInterner;
use dash_middle::parser::error::Error;
use dash_optimizer::type_infer::TypeInferCtx;
use dash_optimizer::OptLevel;
use dash_parser::Parser;

use crate::FunctionCompiler;

impl<'interner> FunctionCompiler<'interner> {
    pub fn compile_str(
        interner: &'interner mut StringInterner,
        input: &str,
        opt: OptLevel,
    ) -> Result<CompileResult, Vec<Error>> {
        let tokens = Lexer::new(interner, input).scan_all()?;
        let (ast, counter) = Parser::new(interner, input, tokens).parse_all()?;

        let tcx = TypeInferCtx::new(counter);

        Self::new(input, opt, tcx, interner)
            .compile_ast(ast, true)
            .map_err(|err| vec![err])
    }
}
