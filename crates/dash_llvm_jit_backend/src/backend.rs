use std::cell::RefCell;
use std::ffi::CStr;
use std::fmt::Debug;
use std::ptr;

use dash_middle::compiler::instruction::Instruction;
use llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm_sys::analysis::LLVMVerifyFunction;
use llvm_sys::analysis::LLVMVerifyModule;
use llvm_sys::core::LLVMCreatePassManager;
use llvm_sys::core::LLVMModuleCreateWithName;
use llvm_sys::core::LLVMPrintModuleToString;
use llvm_sys::core::LLVMRunFunctionPassManager;
use llvm_sys::core::LLVMRunPassManager;
use llvm_sys::execution_engine::LLVMCreateExecutionEngineForModule;
use llvm_sys::execution_engine::LLVMExecutionEngineGetErrMsg;
use llvm_sys::execution_engine::LLVMExecutionEngineRef;
use llvm_sys::execution_engine::LLVMGetFunctionAddress;
use llvm_sys::prelude::LLVMModuleRef;
use llvm_sys::prelude::LLVMPassManagerRef;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::target_machine::LLVMCodeGenOptLevel;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderCreate;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateFunctionPassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateModulePassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderSetOptLevel;

use crate::function::CompileError;
use crate::function::CompileQuery;
use crate::function::Function;
use crate::passes_legacy::infer::InferResult;
use crate::Trace;

pub type JitFunction = unsafe extern "C" fn(
    *mut (),  // stack pointer
    u64,      // stack offset for frame
    *mut u64, // out pointer for the IP after exiting
);

#[macro_export]
macro_rules! cstr {
    ($string:expr) => {
        cstr::cstr!($string).as_ptr()
    };
}

/// The JIT backend.
pub struct Backend {
    cache: RefCell<Vec<Function>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(Vec::new()),
        }
    }

    pub fn compile_trace<Q: CompileQuery>(
        &self,
        q: Q,
        bytecode: &[u8],
        infer: InferResult,
        trace: &Trace,
    ) -> Result<CompiledFunction, CompileError> {
        let mut fun = Function::new();
        fun.compile_setup(&infer.local_tys);
        fun.compile_trace(bytecode, &q, &infer, &trace)?;
        fun.compile_exit_block(&infer.local_tys);

        #[cfg(debug_assertions)]
        fun.verify();

        fun.run_pass_manager();

        let compiled = fun.compile();
        Ok(CompiledFunction { inner: fun, compiled })
    }
}

pub struct CompiledFunction {
    inner: Function,
    compiled: JitFunction,
}

impl CompiledFunction {
    pub fn compiled(&self) -> JitFunction {
        self.compiled
    }
}
