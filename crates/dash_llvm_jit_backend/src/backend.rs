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

use crate::function::CompileQuery;
use crate::function::Function;
use crate::passes::infer::InferResult;
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
    module: LLVMModuleRef,
    engine: LLVMExecutionEngineRef,
    pass_manager: LLVMPassManagerRef,
}

impl Backend {
    pub fn new() -> Self {
        let module = unsafe { LLVMModuleCreateWithName(cstr!("dash_jit")) };
        let mut engine = ptr::null_mut();
        let mut error = ptr::null_mut();
        unsafe { LLVMCreateExecutionEngineForModule(&mut engine, module, &mut error) };
        assert!(!engine.is_null());

        let pass_manager = unsafe {
            let pm = LLVMCreatePassManager();
            let pmb = LLVMPassManagerBuilderCreate();
            LLVMPassManagerBuilderSetOptLevel(pmb, LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive as u32);
            LLVMPassManagerBuilderPopulateFunctionPassManager(pmb, pm);
            LLVMPassManagerBuilderPopulateModulePassManager(pmb, pm);
            pm
        };

        Self {
            module,
            engine,
            pass_manager,
        }
    }

    pub fn compile_trace<Q: CompileQuery>(
        &self,
        q: Q,
        bytecode: &[u8],
        infer: InferResult,
        trace: &Trace,
    ) -> JitFunction {
        let mut fun = Function::new(self);
        fun.compile_setup(&infer.local_tys);
        fun.compile_trace(bytecode, &q, &infer, &trace);
        fun.compile_exit_block(&infer.local_tys);

        #[cfg(debug_assertions)]
        self.verify();

        self.run_pass_manager();

        let fun = self.compile_fn(fun.function_name());
        fun
    }

    pub fn verify(&self) {
        unsafe {
            let mut msg = ptr::null_mut();
            LLVMVerifyModule(self.module, LLVMVerifierFailureAction::LLVMAbortProcessAction, &mut msg);
        }
    }

    pub fn run_pass_manager(&self) {
        unsafe {
            LLVMRunPassManager(self.pass_manager, self.module);
        }
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

    pub fn compile_fn(&self, name: &CStr) -> JitFunction {
        unsafe {
            let addr = LLVMGetFunctionAddress(self.engine, name.as_ptr());
            assert!(addr != 0);
            let fun = std::mem::transmute::<u64, JitFunction>(addr);
            fun
        }
    }
}
