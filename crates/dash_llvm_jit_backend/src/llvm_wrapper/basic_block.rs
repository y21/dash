use llvm_sys::prelude::LLVMBasicBlockRef;

pub struct BasicBlock(pub(super) LLVMBasicBlockRef);
