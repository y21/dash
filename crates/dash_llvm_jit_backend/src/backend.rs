use std::ffi::CStr;
use std::fmt::Debug;
use std::ptr;

use llvm_sys::analysis::LLVMVerifyFunction;
use llvm_sys::core::LLVMModuleCreateWithName;
use llvm_sys::core::LLVMPrintModuleToString;
use llvm_sys::execution_engine::LLVMCreateExecutionEngineForModule;
use llvm_sys::execution_engine::LLVMExecutionEngineRef;
use llvm_sys::prelude::LLVMModuleRef;

use crate::function::CompileQuery;
use crate::function::Function;
use crate::passes::infer::InferResult;
use crate::Trace;

#[macro_export]
macro_rules! cstr {
    ($string:expr) => {
        cstr::cstr!($string).as_ptr()
    };
}

/// The JIT backend.
pub struct Backend {
    module: LLVMModuleRef,
    engine: LLVMExecutionEngineRef,
}

impl Backend {
    pub fn new() -> Self {
        let module = unsafe { LLVMModuleCreateWithName(cstr!("dash_jit")) };
        let mut engine = ptr::null_mut();
        let mut error = ptr::null_mut();
        unsafe { LLVMCreateExecutionEngineForModule(&mut engine, module, &mut error) };
        assert!(!engine.is_null());

        Self { module, engine }
    }

    pub fn compile_trace<Q: CompileQuery>(&self, q: Q, bytecode: &[u8], infer: InferResult, trace: &Trace) {
        let mut fun = Function::new(self);

        fun.init_locals(&infer.local_tys);
        fun.compile_trace(bytecode, q, &infer, &trace);
        self.print_module();
        fun.verify();
    }

    pub fn print_module(&self) {
        let string = unsafe { CStr::from_ptr(LLVMPrintModuleToString(self.module)) };
        let string = String::from_utf8_lossy(string.to_bytes());
        println!("{}", string);
    }

    pub fn module(&self) -> LLVMModuleRef {
        self.module
    }

    pub fn engine(&self) -> LLVMExecutionEngineRef {
        self.engine
    }
}
