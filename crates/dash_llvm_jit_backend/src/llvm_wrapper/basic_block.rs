use llvm_sys::prelude::LLVMBasicBlockRef;

#[derive(Clone)]
pub struct BasicBlock(pub(super) LLVMBasicBlockRef);
