use std::fs;

use dash_compiler::from_string::CompileStrError;
use dash_compiler::FunctionCompiler;
use dash_middle::compiler::StaticImportKind;
use dash_optimizer::OptLevel;
use dash_vm::eval::EvalError;
use dash_vm::frame::Frame;
use dash_vm::local::LocalScope;
use dash_vm::params::VmParams;
use dash_vm::throw;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::Value;
use dash_vm::Vm;
use rand::Rng;
use tokio::sync::mpsc;

use crate::event::EventMessage;
use crate::event::EventSender;
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
            .set_state(Box::new(State::new(rt, EventSender::new(etx))));

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
                    ctx.set_property(&mut scope, "respond".into(), PropertyValue::static_default(fun.into()))
                        .unwrap();

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
            module.set_property(&mut sc, "listen".into(), PropertyValue::static_default(listen.into()))?;

            let module = sc.register(module);
            Ok(module.into())
        }
        "@std/iter" => {
            let module = include_str!("../js/iter.js");
            compile_module(&mut sc, module, import_ty)
        }
        "@std/inspect" => {
            let module = include_str!("../js/inspect.js");
            compile_module(&mut sc, module, import_ty)
        }
        "@std/dl" => {
            #[cfg(feature = "dlopen")]
            {
                dash_dlloader::import_dl(&mut sc)
            }
            #[cfg(not(feature = "dlopen"))]
            {
                throw!(&mut sc, "Dynamic library loading is disabled")
            }
        }
        _ => {
            let contents = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => throw!(&mut sc, "{}", e),
            };

            compile_module(&mut sc, &contents, import_ty)
        }
    }
}

fn compile_module(sc: &mut LocalScope, source: &str, import_ty: StaticImportKind) -> Result<Value, Value> {
    let re = match FunctionCompiler::compile_str(source, OptLevel::Aggressive) {
        Ok(re) => re,
        Err(CompileStrError::Compiler(ce)) => throw!(sc, "Compile error: {:?}", ce),
        Err(CompileStrError::Parser(pe)) => throw!(sc, "Parse error: {:?}", pe),
        Err(CompileStrError::Lexer(le)) => throw!(sc, "Lex error: {:?}", le),
    };

    let frame = Frame::from_compile_result(re);

    let exports = sc.execute_module(frame)?;

    let export_obj = match import_ty {
        StaticImportKind::Default => {
            let export_obj = match exports.default {
                Some(obj) => obj,
                None => {
                    let o = NamedObject::new(sc);
                    Value::Object(sc.register(o))
                }
            };

            export_obj
        }
        StaticImportKind::All => {
            let export_obj = NamedObject::new(sc);

            if let Some(default) = exports.default {
                export_obj.set_property(sc, "default".into(), PropertyValue::static_default(default))?;
            }

            Value::Object(sc.register(export_obj))
        }
    };

    for (k, v) in exports.named {
        export_obj.set_property(sc, String::from(k.as_ref()).into(), PropertyValue::static_default(v))?;
    }

    Ok(export_obj)
}
