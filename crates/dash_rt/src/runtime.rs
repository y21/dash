use std::fmt::Debug;
use std::time::SystemTime;

use dash_middle::compiler::StaticImportKind;
use dash_optimizer::OptLevel;
use dash_vm::eval::EvalError;
use dash_vm::params::VmParams;
use dash_vm::value::string::JsString;
use dash_vm::value::{Unrooted, Value};
use dash_vm::{throw, Vm};
use tokio::sync::mpsc;
use tracing::info;

use crate::event::{EventMessage, EventSender};
use crate::module::ModuleLoader;
use crate::state::State;

#[derive(Debug)]
pub struct Runtime {
    vm: Vm,
    /// Receiver end for the event message channel.
    event_rx: mpsc::UnboundedReceiver<EventMessage>,
}

impl Runtime {
    pub async fn new(initial_gc_threshold: Option<usize>) -> Self {
        let rt = tokio::runtime::Handle::current();

        let (etx, erx) = mpsc::unbounded_channel();

        let state = State::new(rt, EventSender::new(etx));
        let mut params = VmParams::new().set_static_import_callback(import_callback);

        #[cfg(feature = "random")]
        {
            params = params.set_math_random_callback(random_callback);
        }

        params = params
            .set_time_millis_callback(time_callback)
            .set_state(Box::new(state));

        if let Some(threshold) = initial_gc_threshold {
            params = params.set_initial_gc_object_threshold(threshold);
        }

        let vm = Vm::new(params);
        Self { vm, event_rx: erx }
    }

    pub fn vm_params(&mut self) -> &mut VmParams {
        self.vm.params_mut()
    }

    pub fn set_module_manager(&mut self, module_manager: Box<dyn ModuleLoader>) {
        State::from_vm_mut(&mut self.vm).set_root_module(module_manager);
    }

    pub fn eval(&mut self, code: &str, opt: OptLevel) -> Result<Unrooted, EvalError> {
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
                    State::from_vm_mut(&mut self.vm).tasks.remove(id);
                }
            }

            let state = State::from_vm_mut(&mut self.vm);
            if !state.needs_event_loop() {
                info!("Event loop finished");
                return;
            }
        }
    }

    pub fn state_mut(&mut self) -> &mut State {
        State::from_vm_mut(&mut self.vm)
    }

    pub fn state(&self) -> &State {
        State::from_vm(&self.vm)
    }
}

#[cfg(feature = "random")]
fn random_callback(_: &mut Vm) -> Result<f64, Unrooted> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Ok(rng.gen())
}

fn time_callback(_: &mut Vm) -> Result<u64, Unrooted> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time() < UNIX_EPOCH")
        .as_millis() as u64)
}

fn import_callback(vm: &mut Vm, import_ty: StaticImportKind, path: JsString) -> Result<Unrooted, Unrooted> {
    let mut sc = vm.scope();

    let root = State::from_vm_mut(&mut sc).root_module().clone();

    if let Some(module) = &*root.borrow() {
        match module.import(&mut sc, import_ty, path) {
            Ok(Some(module)) => return Ok(module.into()),
            Ok(None) => {}
            Err(err) => return Err(err.into()),
        }
    }

    // If it got here, the module was not found
    let path = path.res(&sc).to_owned();
    throw!(&mut sc, RangeError, "Module not found: {}", path)
}
