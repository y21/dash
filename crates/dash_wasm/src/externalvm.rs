use dash_vm::eval::EvalError;
use dash_vm::frame::Frame;
use dash_vm::local::LocalScope;
use dash_vm::params::VmParams;
use dash_vm::value::Value as DashValue;
use dash_vm::Vm;
use js_sys::Math;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::jsvalue::JsValue;

#[wasm_bindgen]
pub enum OptLevel {
    None,
    Basic,
    Aggressive,
}

impl From<OptLevel> for dash_optimizer::OptLevel {
    fn from(opt_level: OptLevel) -> Self {
        match opt_level {
            OptLevel::None => dash_optimizer::OptLevel::None,
            OptLevel::Basic => dash_optimizer::OptLevel::Basic,
            OptLevel::Aggressive => dash_optimizer::OptLevel::Aggressive,
        }
    }
}

#[derive(Default)]
pub struct ExternalVmState {}

#[wasm_bindgen]
pub struct ExternalVm(Vm);

impl ExternalVm {
    pub fn with_scope<F, T>(&mut self, fun: F) -> T
    where
        F: FnOnce(&mut LocalScope) -> T,
    {
        let mut scope = LocalScope::new(&mut self.0);
        fun(&mut scope)
    }
}

fn math_random(_: &mut Vm) -> Result<f64, DashValue> {
    Ok(Math::random())
}

impl Default for ExternalVm {
    fn default() -> Self {
        let state = ExternalVmState::default();
        let vm = Vm::new(
            VmParams::default()
                .set_math_random_callback(math_random)
                .set_state(Box::new(state)),
        );
        ExternalVm(vm)
    }
}

#[wasm_bindgen]
impl ExternalVm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn global(&self) -> JsValue {
        JsValue::from(DashValue::Object(self.0.global()))
    }

    pub fn eval(&mut self, code: &str, opt: OptLevel) -> Result<JsValue, JsValue> {
        match self.0.eval(code, opt.into()) {
            Ok(value) => Ok(JsValue::from(value)),
            Err(EvalError::Exception(value)) => Err(JsValue::from(value)),
            Err(e) => {
                let err = DashValue::String(format!("{e:?}").into());
                Err(JsValue::from(err))
            }
        }
    }

    pub fn eval_serialized(&mut self, serialized: Uint8Array) -> Result<JsValue, String> {
        let bytecode = serialized.to_vec();
        let deserialized = dash_middle::compiler::format::deserialize(&bytecode).map_err(|e| format!("{e:?}"))?;
        let frame = Frame::from_compile_result(deserialized);
        match self.0.execute_frame(frame) {
            Ok(x) => Ok(JsValue::from(x.into_value())),
            Err(err) => Err(format!("{err:?}")),
        }
    }

    pub fn process_async_tasks(&mut self) {
        self.0.process_async_tasks();
    }
}
