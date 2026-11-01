use std::any::Any;

use dash_middle::compiler::StaticImportKind;
use dash_optimizer::OptLevel;

use crate::UncaughtExceptionSource;
use crate::gc::trace::Trace;
use crate::localscope::LocalScope;
use crate::value::Unrooted;
use crate::value::string::JsString;

use super::Vm;
use super::value::Value;

pub type MathRandomCallback = fn(vm: &mut Vm) -> Result<f64, Unrooted>;
pub type TimeMillisCallback = fn(vm: &mut Vm) -> Result<u64, Unrooted>;
pub type StaticImportCallback = fn(vm: &mut Vm, ty: StaticImportKind, path: JsString) -> Result<Unrooted, Unrooted>;
pub type DynamicImportCallback = fn(vm: &mut Vm, val: Value) -> Result<Unrooted, Unrooted>;
pub type DebuggerCallback = fn(vm: &mut Vm) -> Result<(), Value>;
pub type UnhandledTaskException = fn(vm: &mut LocalScope, exception: Value, source: UncaughtExceptionSource);

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

pub struct VmParams {
    /// Callback that's invoked to generate a random number for `Math.random()`.
    pub math_random_callback: Option<MathRandomCallback>,
    /// Callback that's invoked to get the current time in milliseconds (used by `Date`).
    pub time_millis_callback: Option<TimeMillisCallback>,
    /// Callback that's invoked to resolve static imports.
    pub static_import_callback: Option<StaticImportCallback>,
    /// Callback that's invoked to resolve dynamic imports.
    pub dynamic_import_callback: Option<DynamicImportCallback>,
    /// Callback that's invoked when a `debugger;` statement is evaluated.
    /// No-op if None.
    pub debugger_callback: Option<DebuggerCallback>,
    /// Callback that's invoked when an uncaught exception within an async task is encountered.
    pub unhandled_task_exception_callback: Option<UnhandledTaskException>,
    /// Initial threshold for the garbage collector's RSS before triggering a collection.
    pub initial_gc_rss_threshold: Option<usize>,
    /// Custom state usable by embedders.
    pub state: Option<Box<dyn State>>,
    /// [EXPERIMENTAL] Whether to enable JIT compilation. Enabled by default if compiled with the `jit` feature.
    pub enable_jit: bool,
    /// [EXPERIMENTAL] Dump JIT-compiled code for debugging purposes by passing it through `objdump`.
    pub experimental_objdump_jit: bool,
    /// Optimization level for dynamic code evaluation and JIT.
    pub opt: OptLevel,
}

impl Default for VmParams {
    fn default() -> Self {
        Self {
            math_random_callback: None,
            time_millis_callback: None,
            static_import_callback: None,
            dynamic_import_callback: None,
            debugger_callback: None,
            unhandled_task_exception_callback: None,
            initial_gc_rss_threshold: None,
            state: None,
            enable_jit: true,
            experimental_objdump_jit: false,
            opt: OptLevel::default(),
        }
    }
}

impl VmParams {
    pub fn new() -> Self {
        VmParams::default()
    }

    pub fn state<T: 'static>(&self) -> Option<&T> {
        self.state.as_ref().and_then(|s| (**s).as_any().downcast_ref::<T>())
    }

    pub fn state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state.as_mut().and_then(|s| (**s).as_any_mut().downcast_mut::<T>())
    }
}
