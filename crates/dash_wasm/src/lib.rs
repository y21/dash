use dash_compiler::FunctionCompiler;
use dash_middle::compiler::StaticImportKind;
use dash_middle::parser::statement::VariableDeclarationName;
use dash_optimizer::consteval::Eval;
use dash_optimizer::context::OptimizerContext;
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
    PrettyAst,
    Ast,
}

#[wasm_bindgen]
pub fn evaluate(s: &str, opt: OptLevel, _context: Option<js_sys::Object>) -> Result<String, JsValue> {
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

pub fn fmt_value(value: Value, vm: &mut Vm) -> String {
    let mut scope = LocalScope::new(vm);
    value
        .to_string(&mut scope)
        .map(|s| ToString::to_string(&s))
        .unwrap_or_else(|_| "<exception>".into())
}

#[wasm_bindgen]
pub fn debug(s: &str, o: OptLevel, em: Emit) -> String {
    let parser = Parser::from_str(s).unwrap();
    let mut ast = parser.parse_all().unwrap();

    match em {
        Emit::Bytecode => {
            let cmp = FunctionCompiler::new(o.into()).compile_ast(ast, true).unwrap();
            dash_decompiler::decompile(&cmp.cp, &cmp.instructions).unwrap_or_else(|e| e.to_string())
        }
        Emit::JavaScript => {
            dash_optimizer::optimize_ast(&mut ast, o.into());
            let mut output = String::new();
            for node in ast {
                let _ = write!(output, "{node}\n");
            }
            output
        }
        Emit::Ast => {
            let mut output = String::new();
            for node in ast {
                let _ = write!(output, "{node:?}\n");
            }
            output
        }
        Emit::PrettyAst => {
            let mut output = String::new();
            for node in ast {
                let _ = write!(output, "{node:#?}\n");
            }
            output
        }
    }
}

#[wasm_bindgen]
pub fn compile(s: &str, o: OptLevel) -> Result<js_sys::Uint8Array, String> {
    let cmp = FunctionCompiler::compile_str(s, o.into()).map_err(|e| format!("{e:?}"))?;
    dash_middle::compiler::format::serialize(cmp)
        .map(|v| {
            let u8 = js_sys::Uint8Array::new_with_length(v.len() as u32);
            u8.copy_from(&v);
            u8
        })
        .map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn infer(s: &str) -> Result<JsValue, String> {
    let mut ast = Parser::from_str(s)
        .map_err(|err| format!("{err:?}"))?
        .parse_all()
        .map_err(|err| format!("{err:?}"))?;

    let mut cx = OptimizerContext::new();
    ast.fold(&mut cx, false);
    let mut out = String::new();

    for local in cx.scope_mut().locals() {
        if let VariableDeclarationName::Identifier(ident) = local.binding().name {
            let ty = local.inferred_type().borrow();
            let _ = write!(out, "{ident}: {ty:?} \n");
        }
    }

    Ok(JsValue::from_str(&out))
    // let scope = std::mem::replace(cx.scope_mut(), Scope::new());
    // Ok(JsValue::from_str(&format!("{:?}", scope.locals())))
}

// #[wasm_bindgen]
// pub fn interpret(b: &[u8]) -> Result<String, String> {
//     let cmp = dash_middle::compiler::format::deserialize(b).map_err(|e| format!("{e:?}"))?;
//     let mut vm = Vm::new(VmParams::new());

//     // let cmp = FunctionCompiler::compile_str(s, o.into()).map_err(|e| format!("{e:?}"))?;
//     // dash_middle::compiler::format::serialize(cmp)
//     //     .map(|v| {
//     //         let u8 = js_sys::Uint8Array::new_with_length(v.len() as u32);
//     //         u8.copy_from(&v);
//     //         u8
//     //     })
//     //     .map_err(|e| e.to_string())
// }

fn compile_inspect(vm: &mut Vm) -> Value {
    let source = include_str!("../../dash_rt/js/inspect.js");
    let ast = Parser::from_str(source).unwrap().parse_all().unwrap();
    let re = FunctionCompiler::new(Default::default())
        .compile_ast(ast, true)
        .unwrap();

    let f = Frame::from_compile_result(re);
    vm.execute_module(f).unwrap().default.unwrap()
}
