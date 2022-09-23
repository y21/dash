use std::fmt::Debug;

use dash_middle::compiler::StaticImportKind;
use dash_optimizer::OptLevel;
use dash_vm::eval::EvalError;
use dash_vm::local::LocalScope;
use dash_vm::params::VmParams;
use dash_vm::throw;
use dash_vm::value::Value;
use dash_vm::Vm;
use rand::Rng;
use tokio::sync::mpsc;

use crate::event::EventMessage;
use crate::event::EventSender;
use crate::module::ModuleLoader;
use crate::state::State;

#[derive(Debug)]
pub struct Runtime {
    vm: Vm,
    /// Receiver end for the event message channel.
    event_rx: mpsc::UnboundedReceiver<EventMessage>,
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
            event_rx: erx,
        }
    }

    pub fn set_module_manager(&mut self, module_manager: Box<dyn ModuleLoader>) {
        State::from_vm(&self.vm).set_root_module(module_manager);
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
        while let Some(message) = self.event_rx.recv().await {
            match message {
                EventMessage::ScheduleCallback(fun) => {
                    fun(&mut self);
                }
                EventMessage::RemoveTask(id) => {
                    let tasks = State::from_vm(&self.vm).active_tasks();
                    tasks.remove(id);

                    if !tasks.has_tasks() {
                        // Exit event loop if no more active tasks
                        return;
                    }
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

    let root = State::from_vm(&sc).root_module().clone();

    if let Some(module) = &*root.borrow() {
        if let Some(value) = module.import(&mut sc, import_ty, path) {
            return Ok(value);
        }
    }

    // If it got here, the module was not found
    throw!(sc, "Module not found: {}", path)
}
