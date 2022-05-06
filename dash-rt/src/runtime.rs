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
        // TODO(y21): create Vm::eval
        let tokens = Parser::from_str(code).map_err(EvalError::LexError)?;
        let mut ast = tokens.parse_all().map_err(EvalError::ParseError)?;
        optimizer::optimize_ast(&mut ast, opt);

        let compiled = FunctionCompiler::new()
            .compile_ast(ast)
            .map_err(EvalError::CompileError)?;

        let frame = Frame::from_compile_result(compiled);
        let val = self.vm.execute_frame(frame).map_err(EvalError::VmError)?;
        Ok(val.into_value())
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

fn import_callback(vm: &mut Vm, _ty: StaticImportKind, path: &str) -> Result<Value, Value> {
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
        other => throw!(&mut sc, "Module not found: {}", other),
    }
}
