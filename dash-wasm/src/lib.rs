use dash::compiler::decompiler;
use dash::compiler::FunctionCompiler;
use dash::parser::parser::Parser;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn eval(s: &str) -> String {
    match dash::eval(s) {
        Ok((_vm, value)) => format!("{:?}", value),
        Err(e) => e.to_string().into_owned(),
    }
}

#[wasm_bindgen]
pub fn decompile(s: &str, o: bool) -> String {
    let parser = Parser::from_str(s).unwrap();
    let ast = parser.parse_all(o).unwrap();
    let cmp = FunctionCompiler::compile_ast(ast).unwrap();
    decompiler::decompile(cmp).unwrap_or_else(|e| match e {
        decompiler::DecompileError::AbruptEof => String::from("Error: Abrupt end of file"),
        decompiler::DecompileError::UnknownInstruction(u) => {
            format!("Error: Unknown or unimplemented instruction 0x{:x}", u)
        }
    })
}
