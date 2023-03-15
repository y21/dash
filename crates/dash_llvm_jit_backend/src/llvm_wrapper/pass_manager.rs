use llvm_sys::core::LLVMCreatePassManager;
use llvm_sys::prelude::LLVMPassManagerRef;
use llvm_sys::target_machine::LLVMCodeGenOptLevel;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderCreate;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderDispose;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateFunctionPassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateModulePassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderSetOptLevel;

pub struct PassManager(pub(super) LLVMPassManagerRef);

impl PassManager {
    pub fn new(opt: LLVMCodeGenOptLevel) -> Self {
        unsafe {
            let pm = LLVMCreatePassManager();
            let pmb = LLVMPassManagerBuilderCreate();
            LLVMPassManagerBuilderSetOptLevel(pmb, opt as u32);
            LLVMPassManagerBuilderPopulateFunctionPassManager(pmb, pm);
            LLVMPassManagerBuilderPopulateModulePassManager(pmb, pm);
            LLVMPassManagerBuilderDispose(pmb);

            Self(pm)
        }
    }
}
