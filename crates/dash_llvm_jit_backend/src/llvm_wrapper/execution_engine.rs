use std::ffi::CStr;
use std::mem;

use llvm_sys::execution_engine::LLVMExecutionEngineRef;
use llvm_sys::execution_engine::LLVMGetExecutionEngineTargetData;
use llvm_sys::execution_engine::LLVMGetFunctionAddress;
use llvm_sys::target::LLVMSizeOfTypeInBits;

use super::Ty;

pub type JitFunction = unsafe extern "C" fn(
    *mut (),  // stack pointer
    u64,      // stack offset for frame
    *mut u64, // out pointer for the IP after exiting
);

pub struct ExecutionEngine(pub(super) LLVMExecutionEngineRef);

impl ExecutionEngine {
    pub fn size_of_ty_bits(&self, ty: &Ty) -> usize {
        unsafe {
            // TODO: do we need to free this?
            let target_data = LLVMGetExecutionEngineTargetData(self.0);
            LLVMSizeOfTypeInBits(target_data, ty.0).try_into().unwrap()
        }
    }

    pub fn compile_fn(&self, name: &CStr) -> JitFunction {
        unsafe {
            let addr = LLVMGetFunctionAddress(self.0, name.as_ptr());
            assert!(addr != 0);
            let fun = mem::transmute::<u64, JitFunction>(addr);
            fun
        }
    }
}
