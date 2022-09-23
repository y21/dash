use std::any::Any;

use dash_middle::compiler::StaticImportKind;

use crate::local::LocalScope;

use super::value::Value;
use super::Vm;

pub type MathRandomCallback = fn(vm: &mut Vm) -> Result<f64, Value>;
pub type StaticImportCallback = fn(vm: &mut Vm, ty: StaticImportKind, path: &str) -> Result<Value, Value>;
pub type DynamicImportCallback = fn(vm: &mut Vm, val: Value) -> Result<Value, Value>;
pub type DebuggerCallback = fn(vm: &mut Vm) -> Result<(), Value>;
pub type UnhandledTaskException = fn(vm: &mut LocalScope, exception: Value);

#[derive(Default)]
pub struct VmParams {
    math_random_callback: Option<MathRandomCallback>,
    static_import_callback: Option<StaticImportCallback>,
    dynamic_import_callback: Option<DynamicImportCallback>,
    debugger_callback: Option<DebuggerCallback>,
    unhandled_task_exception_callback: Option<UnhandledTaskException>,
    state: Option<Box<dyn Any>>,
}

impl VmParams {
    pub fn new() -> Self {
        VmParams::default()
    }

    pub fn set_static_import_callback(mut self, callback: StaticImportCallback) -> Self {
        self.static_import_callback = Some(callback);
        self
    }

    pub fn set_dynamic_import_callback(mut self, callback: DynamicImportCallback) -> Self {
        self.dynamic_import_callback = Some(callback);
        self
    }

    pub fn static_import_callback(&self) -> Option<StaticImportCallback> {
        self.static_import_callback
    }

    pub fn dynamic_import_callback(&self) -> Option<DynamicImportCallback> {
        self.dynamic_import_callback
    }

    pub fn set_state(mut self, state: Box<dyn Any>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn update_state(&mut self, state: Box<dyn Any>) {
        self.state = Some(state);
    }

    pub fn state<T: 'static>(&self) -> Option<&T> {
        self.state.as_ref().and_then(|s| s.downcast_ref::<T>())
    }

    pub fn state_raw(&self) -> Option<&dyn Any> {
        self.state.as_deref()
    }

    pub fn set_math_random_callback(mut self, callback: MathRandomCallback) -> Self {
        self.math_random_callback = Some(callback);
        self
    }

    pub fn math_random_callback(&self) -> Option<MathRandomCallback> {
        self.math_random_callback
    }

    pub fn set_debugger_callback(mut self, callback: DebuggerCallback) -> Self {
        self.debugger_callback = Some(callback);
        self
    }

    pub fn debugger_callback(&self) -> Option<DebuggerCallback> {
        self.debugger_callback
    }

    pub fn set_unhandled_task_exception_callback(mut self, callback: UnhandledTaskException) -> Self {
        self.unhandled_task_exception_callback = Some(callback);
        self
    }

    pub fn unhandled_task_exception_callback(&self) -> Option<UnhandledTaskException> {
        self.unhandled_task_exception_callback
    }
}
