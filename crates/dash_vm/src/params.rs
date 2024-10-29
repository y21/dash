use std::any::Any;

use dash_middle::compiler::StaticImportKind;

use crate::gc::trace::Trace;
use crate::localscope::LocalScope;
use crate::value::string::JsString;
use crate::value::Unrooted;

use super::value::Value;
use super::Vm;

pub type MathRandomCallback = fn(vm: &mut Vm) -> Result<f64, Unrooted>;
pub type TimeMillisCallback = fn(vm: &mut Vm) -> Result<u64, Unrooted>;
pub type StaticImportCallback = fn(vm: &mut Vm, ty: StaticImportKind, path: JsString) -> Result<Unrooted, Unrooted>;
pub type DynamicImportCallback = fn(vm: &mut Vm, val: Value) -> Result<Unrooted, Unrooted>;
pub type DebuggerCallback = fn(vm: &mut Vm) -> Result<(), Value>;
pub type UnhandledTaskException = fn(vm: &mut LocalScope, exception: Value);

pub trait State: Any + Trace {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl<T: Any + Trace> State for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct VmParams {
    pub math_random_callback: Option<MathRandomCallback>,
    pub time_millis_callback: Option<TimeMillisCallback>,
    pub static_import_callback: Option<StaticImportCallback>,
    pub dynamic_import_callback: Option<DynamicImportCallback>,
    pub debugger_callback: Option<DebuggerCallback>,
    pub unhandled_task_exception_callback: Option<UnhandledTaskException>,
    pub initial_gc_rss_threshold: Option<usize>,
    pub state: Option<Box<dyn State>>,
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

    pub fn set_state(mut self, state: Box<dyn State>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn update_state(&mut self, state: Box<dyn State>) {
        self.state = Some(state);
    }

    pub fn state<T: 'static>(&self) -> Option<&T> {
        self.state.as_ref().and_then(|s| (**s).as_any().downcast_ref::<T>())
    }

    pub fn state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state.as_mut().and_then(|s| (**s).as_any_mut().downcast_mut::<T>())
    }

    pub fn state_raw(&self) -> Option<&dyn State> {
        self.state.as_deref()
    }

    pub fn set_math_random_callback(mut self, callback: MathRandomCallback) -> Self {
        self.math_random_callback = Some(callback);
        self
    }
    pub fn set_time_millis_callback(mut self, callback: TimeMillisCallback) -> Self {
        self.time_millis_callback = Some(callback);
        self
    }

    pub fn set_debugger_callback(mut self, callback: DebuggerCallback) -> Self {
        self.debugger_callback = Some(callback);
        self
    }

    pub fn set_unhandled_task_exception_callback(mut self, callback: UnhandledTaskException) -> Self {
        self.unhandled_task_exception_callback = Some(callback);
        self
    }

    pub fn unhandled_task_exception_callback(&self) -> Option<UnhandledTaskException> {
        self.unhandled_task_exception_callback
    }

    pub fn set_initial_gc_rss_threshold(mut self, threshold: usize) -> Self {
        self.initial_gc_rss_threshold = Some(threshold);
        self
    }
}
