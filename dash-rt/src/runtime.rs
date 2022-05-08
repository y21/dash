use std::fs;

use dash_core::compiler::FunctionCompiler;
use dash_core::compiler::StaticImportKind;
use dash_core::optimizer;
use dash_core::optimizer::consteval::OptLevel;
use dash_core::parser::parser::Parser;
use dash_core::throw;
use dash_core::vm::frame::Frame;
use dash_core::vm::local::LocalScope;
use dash_core::vm::params::VmParams;
use dash_core::vm::value::function::Function;
use dash_core::vm::value::function::FunctionKind;
use dash_core::vm::value::object::NamedObject;
use dash_core::vm::value::object::Object;
use dash_core::vm::value::Value;
use dash_core::vm::Vm;
use dash_core::EvalError;
use rand::Rng;
use tokio::sync::mpsc;

use crate::event::EventMessage;
use crate::http;
use crate::http::HttpContext;
use crate::state::State;

#[derive(Debug)]
pub struct Runtime {
    vm: Vm,
    erx: mpsc::UnboundedReceiver<EventMessage>,
}

impl Runtime {
    pub async fn new() -> Self {
        let rt = tokio::runtime::Handle::current();

        let (etx, erx) = mpsc::unbounded_channel();

        let params = VmParams::new()
            .set_static_import_callback(import_callback)
            .set_math_random_callback(random_callback)
            .set_state(Box::new(State::new(rt, etx)));

        Self {
            vm: Vm::new(params),
            erx,
        }
    }

    pub fn eval<'i>(&mut self, code: &'i str, opt: OptLevel) -> Result<Value, EvalError<'i>> {
        self.vm.eval(code, opt)
    }

    pub fn vm(&self) -> &Vm {
        &self.vm
    }

    pub fn vm_mut(&mut self) -> &mut Vm {
        &mut self.vm
    }

    pub async fn run_event_loop(mut self) {
        while let Some(message) = self.erx.recv().await {
            match message {
                EventMessage::HttpRequest(_, ttx) => {
                    let mut scope = LocalScope::new(&mut self.vm);
                    let state = State::try_from_vm(&scope).unwrap();
                    let cb = match state.http_handler() {
                        Some(cb) => cb,
                        None => continue,
                    };

                    let ctx = HttpContext::new(&mut scope, ttx);
                    let fun = Function::new(
                        &mut scope,
                        Some("respond".into()),
                        FunctionKind::Native(http::ctx_respond),
                    );
                    let fun = scope.register(fun);
                    ctx.set_property(&mut scope, "respond".into(), fun.into()).unwrap();

                    let ctx = Value::Object(scope.register(ctx));

                    // TODO(y21): do not unwrap
                    cb.apply(&mut scope, Value::undefined(), vec![ctx]).unwrap();
                }
            }
        }
    }
}

fn random_callback(_: &mut Vm) -> Result<f64, Value> {
    let mut rng = rand::thread_rng();
    Ok(rng.gen())
}

fn import_callback(vm: &mut Vm, import_ty: StaticImportKind, path: &str) -> Result<Value, Value> {
    let mut sc = LocalScope::new(vm);

    match path {
        "@std/http" => {
            let module = NamedObject::new(&mut sc);
            let listen = Function::new(&mut sc, None, FunctionKind::Native(http::listen));
            let listen = sc.register(listen);
            module.set_property(&mut sc, "listen".into(), listen.into())?;

            let module = sc.register(module);
            Ok(module.into())
        }
        _ => {
            let contents = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => throw!(&mut sc, "{}", e),
            };

            let tokens = match Parser::from_str(&contents) {
                Ok(tok) => tok,
                Err(e) => throw!(&mut sc, "Module lex error: {:?}", e),
            };

            let mut ast = match tokens.parse_all() {
                Ok(ast) => ast,
                Err(e) => throw!(&mut sc, "Module parse error: {:?}", e),
            };

            optimizer::optimize_ast(&mut ast, OptLevel::Aggressive);

            let re = match FunctionCompiler::new().compile_ast(ast) {
                Ok(re) => re,
                Err(e) => throw!(&mut sc, "Module compile error: {:?}", e),
            };

            let frame = Frame::from_compile_result(re);

            let exports = sc.execute_module(frame)?;

            let export_obj = match import_ty {
                StaticImportKind::Default => {
                    let export_obj = match exports.default {
                        Some(obj) => obj,
                        None => {
                            let o = NamedObject::new(&mut sc);
                            Value::Object(sc.register(o))
                        }
                    };

                    export_obj
                }
                StaticImportKind::All => {
                    let export_obj = NamedObject::new(&mut sc);

                    if let Some(default) = exports.default {
                        export_obj.set_property(&mut sc, "default".into(), default)?;
                    }

                    Value::Object(sc.register(export_obj))
                }
            };

            for (k, v) in exports.named {
                export_obj.set_property(&mut sc, String::from(k.as_ref()).into(), v)?;
            }

            Ok(export_obj)
        }
    }
}
