use std::slice;

use llvm_sys::core::LLVMGetTypeKind;
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::LLVMTypeKind;

use crate::util::transmute_slice_mut;

#[repr(transparent)]
pub struct Ty(pub(super) LLVMTypeRef);

impl Ty {
    pub fn slice_of_tys_as_raw(slice: &mut [Ty]) -> &mut [LLVMTypeRef] {
        unsafe { transmute_slice_mut(slice) }
    }

    pub fn kind(&self) -> LLVMTypeKind {
        unsafe { LLVMGetTypeKind(self.0) }
    }
}
