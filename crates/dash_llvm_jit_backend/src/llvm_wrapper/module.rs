use std::ffi::CStr;
use std::ptr;

use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyModule};
use llvm_sys::core::{
    LLVMAddFunction, LLVMDisposeMessage, LLVMPrintModuleToString, LLVMRunPassManager, LLVMSetInstructionCallConv,
};
use llvm_sys::execution_engine::LLVMCreateExecutionEngineForModule;
use llvm_sys::prelude::LLVMModuleRef;
use llvm_sys::LLVMCallConv;

use super::execution_engine::ExecutionEngine;
use super::{Function, PassManager, Ty, Value};

pub struct Module(pub(super) LLVMModuleRef);

impl Module {
    pub fn create_execution_engine(&self) -> ExecutionEngine {
        let mut engine = ptr::null_mut();
        let mut err = ptr::null_mut();
        assert!(unsafe { LLVMCreateExecutionEngineForModule(&mut engine, self.0, &mut err) == 0 });
        assert!(err.is_null());
        ExecutionEngine(engine)
    }

    /// Creates a function with a given name using the C calling convention
    pub fn create_c_function_with_name(&self, name: &CStr, ty: &Ty) -> Function {
        unsafe {
            let function = LLVMAddFunction(self.0, name.as_ptr(), ty.0);
            LLVMSetInstructionCallConv(function, LLVMCallConv::LLVMCCallConv as u32);
            Function(Value(function))
        }
    }

    pub fn create_c_function(&self, ty: &Ty) -> Function {
        self.create_c_function_with_name(c"anon", ty)
    }

    pub fn run_pass_manager(&self, pm: &PassManager) {
        unsafe { LLVMRunPassManager(pm.0, self.0) };
    }

    pub fn print_module(&self) {
        let string = unsafe { CStr::from_ptr(LLVMPrintModuleToString(self.0)) };
        let rust_string = String::from_utf8_lossy(string.to_bytes());
        println!("{rust_string}");

        unsafe { LLVMDisposeMessage(string.as_ptr() as *mut i8) }
    }

    pub fn verify(&self) {
        let mut error = ptr::null_mut();
        unsafe {
            LLVMVerifyModule(self.0, LLVMVerifierFailureAction::LLVMAbortProcessAction, &mut error);
            LLVMDisposeMessage(error);
        }
    }
}
