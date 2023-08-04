use std::collections::HashMap;

use dash_llvm_jit_backend::codegen;
use dash_llvm_jit_backend::codegen::JitFunction;
use dash_llvm_jit_backend::error::Error;
use dash_llvm_jit_backend::init;
use dash_middle::compiler::constant::Function;
use dash_typed_cfg::TypedCfg;

use crate::Vm;

use super::query::QueryProvider;
pub use dash_llvm_jit_backend::Trace;

pub struct Frontend {
    /// If we are currently recording a trace for a loop iteration,
    /// this will contain metadata such as the pc of the loop header and its end
    trace: Option<Trace>,
    cache: HashMap<CacheKey, (TypedCfg, JitFunction)>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub origin: *const Function,
    pub ip: usize,
}

impl CacheKey {
    pub fn from_trace(trace: &Trace) -> Self {
        Self {
            ip: trace.start(),
            origin: trace.origin(),
        }
    }
}

impl Frontend {
    pub fn new() -> Self {
        init();

        Self {
            trace: None,
            cache: HashMap::new(),
        }
    }

    pub fn recording_trace(&self) -> Option<&Trace> {
        self.trace.as_ref()
    }

    pub fn recording_trace_mut(&mut self) -> Option<&mut Trace> {
        self.trace.as_mut()
    }

    pub fn take_recording_trace(&mut self) -> Option<Trace> {
        self.trace.take()
    }

    pub fn set_recording_trace(&mut self, trace: Trace) {
        self.trace = Some(trace);
    }
}

pub fn compile_current_trace(vm: &mut Vm) -> Result<(Trace, JitFunction), Error> {
    let frame = vm.frames.last().unwrap();
    let trace = vm.jit.take_recording_trace().unwrap();
    let bytecode = frame
        .function
        .buffer
        .with(|buf| buf[trace.start()..trace.end()].to_vec()); // We can do better than cloning, but good enough for now.

    let key = CacheKey::from_trace(&trace);

    // only check cache if we are allowed to.
    // if not, recompile and recache.
    let allow_cache = !trace.is_subtrace();
    if allow_cache {
        if let Some((_, fun)) = vm.jit.cache.get(&key) {
            return Ok((trace, *fun));
        }
    }

    let mut query = QueryProvider { vm, trace: &trace };
    let tcfg = dash_typed_cfg::lower(&bytecode, &mut query)?;
    let fun = codegen::compile_typed_cfg(&bytecode, &tcfg, &mut query)?;

    vm.jit.cache.insert(key, (tcfg, fun));

    Ok((trace, fun))
}
