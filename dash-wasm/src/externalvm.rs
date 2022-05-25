use dash_core::vm::local::LocalScope;
use dash_core::vm::params::VmParams;
use dash_core::vm::value::Value as DashValue;
use dash_core::vm::Vm;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::jsvalue::JsValue;

#[wasm_bindgen]
pub enum OptLevel {
    None,
    Basic,
    Aggressive,
}

impl From<OptLevel> for dash_core::optimizer::consteval::OptLevel {
    fn from(opt_level: OptLevel) -> Self {
        match opt_level {
            OptLevel::None => dash_core::optimizer::consteval::OptLevel::None,
            OptLevel::Basic => dash_core::optimizer::consteval::OptLevel::Basic,
            OptLevel::Aggressive => dash_core::optimizer::consteval::OptLevel::Aggressive,
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

#[wasm_bindgen]
impl ExternalVm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let state = ExternalVmState::default();
        let vm = Vm::new(VmParams::default().set_state(Box::new(state)));
        ExternalVm(vm)
    }

    pub fn global(&self) -> JsValue {
        JsValue::from(DashValue::Object(self.0.global()))
    }

    pub fn eval(&mut self, code: &str, opt: OptLevel) -> Result<JsValue, String> {
        match self.0.eval(code, opt.into()) {
            Ok(value) => Ok(JsValue::from(value)),
            Err(e) => Err(format!("{}", e)), // TODO: use inspect?
        }
    }
}
