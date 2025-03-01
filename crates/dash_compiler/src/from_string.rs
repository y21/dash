use dash_lexer::Lexer;
use dash_middle::compiler::CompileResult;
use dash_middle::interner::StringInterner;
use dash_middle::parser::error::Error;
use dash_optimizer::OptLevel;
use dash_optimizer::type_infer::name_res;
use dash_parser::Parser;

use crate::FunctionCompiler;

impl<'interner> FunctionCompiler<'interner> {
    pub fn compile_str(
        interner: &'interner mut StringInterner,
        input: &str,
        opt: OptLevel,
    ) -> Result<CompileResult, Vec<Error>> {
        let tokens = Lexer::new(interner, input).scan_all()?;
        let (ast, scope_counter, local_counter) = Parser::new(interner, input, tokens).parse_all()?;

        let nameres = name_res(&ast, scope_counter.len(), local_counter.len());

        Self::new(input, opt, nameres, scope_counter, interner)
            .compile_ast(ast, true)
            .map_err(|err| vec![err])
    }
}
