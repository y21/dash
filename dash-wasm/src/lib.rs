use dash::compiler::decompiler;
use dash::compiler::FunctionCompiler;
use dash::optimizer;
use dash::parser::parser::Parser;
use dash::vm::value::Value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn eval(s: &str) -> String {
    match dash::eval(s) {
        Ok((_vm, value)) => match value {
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::String(s) => s.to_string(),
            Value::Null => String::from("null"),
            Value::Undefined => String::from("undefined"),
            Value::Object(o) => format!("{:?}", o.as_ptr()),
        },
        Err(e) => e.to_string(),
    }
}

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
