use llvm_sys::core::LLVMAddIncoming;
use llvm_sys::core::LLVMBuildAdd;
use llvm_sys::core::LLVMBuildAlloca;
use llvm_sys::core::LLVMBuildBitCast;
use llvm_sys::core::LLVMBuildGEP2;
use llvm_sys::core::LLVMBuildLoad2;
use llvm_sys::core::LLVMBuildPhi;
use llvm_sys::core::LLVMBuildRetVoid;
use llvm_sys::core::LLVMBuildSExt;
use llvm_sys::core::LLVMBuildSIToFP;
use llvm_sys::core::LLVMBuildStore;
use llvm_sys::core::LLVMBuildTrunc;
use llvm_sys::core::LLVMPositionBuilderAtEnd;
use llvm_sys::prelude::LLVMBuilderRef;

use crate::cstrp;

use super::value::Phi;
use super::BasicBlock;
use super::Ty;
use super::Value;

pub struct Builder(pub(super) LLVMBuilderRef);

impl Builder {
    pub fn position_at_end(&self, bb: &BasicBlock) {
        unsafe { LLVMPositionBuilderAtEnd(self.0, bb.0) }
    }

    /// Creates an alloca instruction for a local variable
    pub fn build_alloca(&self, ty: &Ty) -> Value {
        Value(unsafe { LLVMBuildAlloca(self.0, ty.0, cstrp!("alloca")) })
    }

    pub fn build_add(&self, a: &Value, b: &Value) -> Value {
        Value(unsafe { LLVMBuildAdd(self.0, a.0, b.0, cstrp!("add")) })
    }

    pub fn build_gep(&self, ty: &Ty, val: &Value, indices: &mut [Value]) -> Value {
        let indices = Value::slice_of_values_as_raw(indices);
        Value(unsafe {
            LLVMBuildGEP2(
                self.0,
                ty.0,
                val.0,
                indices.as_mut_ptr(),
                indices.len().try_into().unwrap(),
                cstrp!("gep"),
            )
        })
    }

    pub fn build_load(&self, ty: &Ty, val: &Value) -> Value {
        Value(unsafe { LLVMBuildLoad2(self.0, ty.0, val.0, cstrp!("load")) })
    }

    pub fn build_trunc(&self, ty: &Ty, value: &Value) -> Value {
        Value(unsafe { LLVMBuildTrunc(self.0, value.0, ty.0, cstrp!("trunc")) })
    }

    pub fn build_sext(&self, ty: &Ty, value: &Value) -> Value {
        Value(unsafe { LLVMBuildSExt(self.0, value.0, ty.0, cstrp!("sext")) })
    }

    pub fn build_si2fp(&self, ty: &Ty, value: &Value) -> Value {
        Value(unsafe { LLVMBuildSIToFP(self.0, value.0, ty.0, cstrp!("si2fp")) })
    }

    pub fn build_fp2si(&self, ty: &Ty, value: &Value) -> Value {
        Value(unsafe { LLVMBuildSIToFP(self.0, value.0, ty.0, cstrp!("fp2si")) })
    }

    pub fn build_bitcast(&self, to: &Ty, value: &Value) -> Value {
        Value(unsafe { LLVMBuildBitCast(self.0, value.0, to.0, cstrp!("bitcast")) })
    }

    pub fn build_store(&self, value: &Value, dest: &Value) -> Value {
        Value(unsafe { LLVMBuildStore(self.0, value.0, dest.0) })
    }

    pub fn build_phi(&self, ty: &Ty) -> Phi {
        Phi(Value(unsafe { LLVMBuildPhi(self.0, ty.0, cstrp!("phi")) }))
    }

    pub fn build_retvoid(&self) -> Value {
        Value(unsafe { LLVMBuildRetVoid(self.0) })
    }
}
