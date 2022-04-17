use dash_core as dash;

use dash::compiler::decompiler;
use dash::compiler::FunctionCompiler;
use dash::optimizer;
use dash::parser::parser::Parser;
use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;
use dash::vm::value::Value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum OptLevel {
    None,
    Basic,
    Aggressive,
}

impl From<OptLevel> for dash::optimizer::consteval::OptLevel {
    fn from(opt_level: OptLevel) -> Self {
        match opt_level {
            OptLevel::None => dash::optimizer::consteval::OptLevel::None,
            OptLevel::Basic => dash::optimizer::consteval::OptLevel::Basic,
            OptLevel::Aggressive => dash::optimizer::consteval::OptLevel::Aggressive,
        }
    }
}

#[wasm_bindgen]
pub fn eval(s: &str, opt: OptLevel) -> String {
    match dash::eval(s, opt.into()) {
        Ok((mut vm, value)) => match value {
            Value::External(e) => format!("[external@{:?}]", e.as_ptr()),
            other => {
                let mut scope = LocalScope::new(&mut vm);

                // TODO: add value to scope
                other
                    .to_string(&mut scope)
                    .map(|x| x.to_string())
                    .unwrap_or_else(|_| "<exception>".into())
            }
        },
        Err(e) => e.to_string(),
    }
}

#[wasm_bindgen]
pub fn decompile(s: &str, o: OptLevel) -> String {
    let parser = Parser::from_str(s).unwrap();
    let mut ast = parser.parse_all().unwrap();
    optimizer::optimize_ast(&mut ast, o.into());
    let cmp = FunctionCompiler::new().compile_ast(ast).unwrap();
    decompiler::decompile(cmp).unwrap_or_else(|e| match e {
        decompiler::DecompileError::AbruptEof => String::from("Error: Abrupt end of file"),
        decompiler::DecompileError::UnknownInstruction(u) => {
            format!("Error: Unknown or unimplemented instruction 0x{:x}", u)
        }
    })
}
