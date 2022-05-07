use dash::compiler::StaticImportKind;
use dash::vm::params::VmParams;
use dash::vm::Vm;
use dash_core as dash;

use dash::compiler::decompiler;
use dash::compiler::FunctionCompiler;
use dash::optimizer;
use dash::parser::parser::Parser;
use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;
use dash::vm::value::Value;
use std::fmt::Write;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum OptLevel {
    None,
    Basic,
    Aggressive,
}

#[wasm_bindgen]
pub enum Emit {
    Bytecode,
    JavaScript,
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
    fn import_callback(_: &mut Vm, _: StaticImportKind, path: &str) -> Result<Value, Value> {
        Ok(Value::String(format!("Hello from module {path}").into()))
    }

    fn random_callback(_: &mut Vm) -> Result<f64, Value> {
        Ok(js_sys::Math::random())
    }

    let params = VmParams::new()
        .set_static_import_callback(import_callback)
        .set_math_random_callback(random_callback);

    let mut vm = Vm::new(params);

    match vm.eval(s, opt.into()) {
        Ok(value) => match value {
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
pub fn decompile(s: &str, o: OptLevel, em: Emit) -> String {
    let parser = Parser::from_str(s).unwrap();
    let mut ast = parser.parse_all().unwrap();
    optimizer::optimize_ast(&mut ast, o.into());

    match em {
        Emit::Bytecode => {
            let cmp = FunctionCompiler::new().compile_ast(ast).unwrap();
            decompiler::decompile(cmp).unwrap_or_else(|e| match e {
                decompiler::DecompileError::AbruptEof => String::from("Error: Abrupt end of file"),
                decompiler::DecompileError::UnknownInstruction(u) => {
                    format!("Error: Unknown or unimplemented instruction 0x{:x}", u)
                }
            })
        }
        Emit::JavaScript => {
            let mut output = String::new();
            for node in ast {
                let _ = write!(output, "{node}; ");
            }
            output
        }
    }
}
