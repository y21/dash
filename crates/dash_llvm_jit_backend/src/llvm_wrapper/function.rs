use std::ffi::CStr;

use llvm_sys::core::LLVMGetParam;
use llvm_sys::core::LLVMGetValueName2;
use llvm_sys::prelude::LLVMValueRef;

use super::value::Value;

pub struct Function(pub(super) Value);

impl Function {
    pub fn as_ptr(&self) -> LLVMValueRef {
        self.0 .0
    }

    pub fn get_param(&self, param: u32) -> Value {
        Value(unsafe { LLVMGetParam(self.as_ptr(), param) })
    }

    pub fn name(&self) -> &CStr {
        unsafe {
            // TODO: is this correct? what is length even used for?
            let mut length = 0;
            let name = LLVMGetValueName2(self.as_ptr(), &mut length);
            let name = CStr::from_ptr(name);
            name
        }
    }
}
