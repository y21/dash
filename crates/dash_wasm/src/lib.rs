use dash_compiler::decompiler;
use dash_compiler::decompiler::DecompileError;
use dash_compiler::FunctionCompiler;
use dash_middle::compiler::StaticImportKind;
use dash_parser::Parser;
use dash_vm::eval::EvalError;
use dash_vm::frame::Frame;
use dash_vm::local::LocalScope;
use dash_vm::params::VmParams;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::value::Value;
use dash_vm::Vm;
use std::fmt::Write;
use wasm_bindgen::prelude::*;

use crate::externalvm::OptLevel;

mod externalfunction;
mod externalvm;
mod jsvalue;
mod util;

#[wasm_bindgen]
pub enum Emit {
    Bytecode,
    JavaScript,
}

#[wasm_bindgen]
pub fn eval(s: &str, opt: OptLevel, _context: Option<js_sys::Object>) -> Result<String, JsValue> {
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

    let result = match vm.eval(s, opt.into()) {
        Ok(value) => {
            let mut scope = LocalScope::new(&mut vm);
            let inspect = compile_inspect(&mut scope);

            let value = inspect
                .apply(&mut scope, Value::undefined(), vec![value])
                .map(|x| match x {
                    Value::String(s) => String::from(s.as_ref()),
                    _ => unreachable!(),
                });

            match value {
                Ok(value) => value,
                Err(e) => fmt_value(e, &mut scope),
            }
        }
        Err(EvalError::Exception(val)) => fmt_value(val, &mut vm),
        Err(e) => format!("{e:?}").into(),
    };

    Ok(result)
}

fn fmt_value(value: Value, vm: &mut Vm) -> String {
    let mut scope = LocalScope::new(vm);
    value
        .to_string(&mut scope)
        .map(|s| ToString::to_string(&s))
        .unwrap_or_else(|_| "<exception>".into())
}

#[wasm_bindgen]
pub fn decompile(s: &str, o: OptLevel, em: Emit) -> String {
    let parser = Parser::from_str(s).unwrap();
    let mut ast = parser.parse_all().unwrap();

    match em {
        Emit::Bytecode => {
            let cmp = FunctionCompiler::new(o.into()).compile_ast(ast, true).unwrap();
            decompiler::decompile(cmp).unwrap_or_else(|e| match e {
                DecompileError::AbruptEof => String::from("Error: Abrupt end of file"),
                DecompileError::UnknownInstruction(u) => {
                    format!("Error: Unknown instruction 0x{:x}", u)
                }
                DecompileError::UnimplementedInstruction(i) => {
                    format!("Error: Unimplemented instruction {:?}", i)
                }
            })
        }
        Emit::JavaScript => {
            dash_optimizer::optimize_ast(&mut ast, o.into());
            let mut output = String::new();
            for node in ast {
                let _ = write!(output, "{node}; ");
            }
            output
        }
    }
}

fn compile_inspect(vm: &mut Vm) -> Value {
    let source = include_str!("../../dash_rt/js/inspect.js");
    let ast = Parser::from_str(source).unwrap().parse_all().unwrap();
    let re = FunctionCompiler::new(Default::default())
        .compile_ast(ast, true)
        .unwrap();

    let f = Frame::from_compile_result(re);
    vm.execute_module(f).unwrap().default.unwrap()
}
