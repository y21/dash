use llvm_sys::core::LLVMAddIncoming;
use llvm_sys::core::LLVMBuildAdd;
use llvm_sys::core::LLVMBuildAlloca;
use llvm_sys::core::LLVMBuildBitCast;
use llvm_sys::core::LLVMBuildBr;
use llvm_sys::core::LLVMBuildCondBr;
use llvm_sys::core::LLVMBuildFAdd;
use llvm_sys::core::LLVMBuildFCmp;
use llvm_sys::core::LLVMBuildFDiv;
use llvm_sys::core::LLVMBuildFMul;
use llvm_sys::core::LLVMBuildFPToSI;
use llvm_sys::core::LLVMBuildFRem;
use llvm_sys::core::LLVMBuildFSub;
use llvm_sys::core::LLVMBuildGEP2;
use llvm_sys::core::LLVMBuildICmp;
use llvm_sys::core::LLVMBuildLoad2;
use llvm_sys::core::LLVMBuildMul;
use llvm_sys::core::LLVMBuildPhi;
use llvm_sys::core::LLVMBuildRetVoid;
use llvm_sys::core::LLVMBuildSDiv;
use llvm_sys::core::LLVMBuildSExt;
use llvm_sys::core::LLVMBuildSIToFP;
use llvm_sys::core::LLVMBuildSRem;
use llvm_sys::core::LLVMBuildStore;
use llvm_sys::core::LLVMBuildSub;
use llvm_sys::core::LLVMBuildTrunc;
use llvm_sys::core::LLVMPositionBuilderAtEnd;
use llvm_sys::prelude::LLVMBuilderRef;
use llvm_sys::LLVMIntPredicate;
use llvm_sys::LLVMRealPredicate;
use llvm_sys::LLVMTypeKind;

use crate::cstrp;

use super::value::Phi;
use super::BasicBlock;
use super::Ty;
use super::Value;

pub enum Predicate {
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}

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
        let ty = a.ty_kind();
        let res = unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildAdd(self.0, a.0, b.0, cstrp!("iadd")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFAdd(self.0, a.0, b.0, cstrp!("fadd")),
                _ => panic!("unsupported type"),
            }
        };
        Value(res)
    }

    pub fn build_sub(&self, a: &Value, b: &Value) -> Value {
        let ty = a.ty_kind();
        let res = unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildSub(self.0, a.0, b.0, cstrp!("isub")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFSub(self.0, a.0, b.0, cstrp!("fsub")),
                _ => panic!("unsupported type"),
            }
        };
        Value(res)
    }

    pub fn build_mul(&self, a: &Value, b: &Value) -> Value {
        let ty = a.ty_kind();
        let res = unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildMul(self.0, a.0, b.0, cstrp!("imul")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFMul(self.0, a.0, b.0, cstrp!("fmul")),
                _ => panic!("unsupported type"),
            }
        };
        Value(res)
    }

    pub fn build_div(&self, a: &Value, b: &Value) -> Value {
        let ty = a.ty_kind();
        let res = unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildSDiv(self.0, a.0, b.0, cstrp!("idiv")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFDiv(self.0, a.0, b.0, cstrp!("fdiv")),
                _ => panic!("unsupported type"),
            }
        };
        Value(res)
    }

    pub fn build_rem(&self, a: &Value, b: &Value) -> Value {
        let ty = a.ty_kind();
        Value(unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildSRem(self.0, a.0, b.0, cstrp!("srem")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFRem(self.0, a.0, b.0, cstrp!("frem")),
                _ => panic!("unsupported type"),
            }
        })
    }

    pub fn build_cmp(&self, a: &Value, b: &Value, pred: Predicate) -> Value {
        let ty = a.ty_kind();
        Value(unsafe {
            match (ty, pred) {
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Le) => {
                    LLVMBuildICmp(self.0, LLVMIntPredicate::LLVMIntSLE, a.0, b.0, cstrp!("ile"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Lt) => {
                    LLVMBuildICmp(self.0, LLVMIntPredicate::LLVMIntSLT, a.0, b.0, cstrp!("ilt"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Ge) => {
                    LLVMBuildICmp(self.0, LLVMIntPredicate::LLVMIntSGE, a.0, b.0, cstrp!("ige"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Gt) => {
                    LLVMBuildICmp(self.0, LLVMIntPredicate::LLVMIntSGT, a.0, b.0, cstrp!("igt"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Eq) => {
                    LLVMBuildICmp(self.0, LLVMIntPredicate::LLVMIntEQ, a.0, b.0, cstrp!("ieq"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Ne) => {
                    LLVMBuildICmp(self.0, LLVMIntPredicate::LLVMIntNE, a.0, b.0, cstrp!("ine"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Le) => {
                    LLVMBuildFCmp(self.0, LLVMRealPredicate::LLVMRealULE, a.0, b.0, cstrp!("fle"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Lt) => {
                    LLVMBuildFCmp(self.0, LLVMRealPredicate::LLVMRealULT, a.0, b.0, cstrp!("flt"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Ge) => {
                    LLVMBuildFCmp(self.0, LLVMRealPredicate::LLVMRealUGE, a.0, b.0, cstrp!("fge"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Gt) => {
                    LLVMBuildFCmp(self.0, LLVMRealPredicate::LLVMRealUGT, a.0, b.0, cstrp!("fgt"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Eq) => {
                    LLVMBuildFCmp(self.0, LLVMRealPredicate::LLVMRealUEQ, a.0, b.0, cstrp!("feq"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Ne) => {
                    LLVMBuildFCmp(self.0, LLVMRealPredicate::LLVMRealUNE, a.0, b.0, cstrp!("fne"))
                }
                _ => panic!("Unsupported type for comparison"),
            }
        })
    }

    pub fn build_lt(&self, a: &Value, b: &Value) -> Value {
        self.build_cmp(a, b, Predicate::Lt)
    }

    pub fn build_gt(&self, a: &Value, b: &Value) -> Value {
        self.build_cmp(a, b, Predicate::Gt)
    }

    pub fn build_le(&self, a: &Value, b: &Value) -> Value {
        self.build_cmp(a, b, Predicate::Le)
    }

    pub fn build_ge(&self, a: &Value, b: &Value) -> Value {
        self.build_cmp(a, b, Predicate::Ge)
    }

    pub fn build_eq(&self, a: &Value, b: &Value) -> Value {
        self.build_cmp(a, b, Predicate::Eq)
    }

    pub fn build_ne(&self, a: &Value, b: &Value) -> Value {
        self.build_cmp(a, b, Predicate::Ne)
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
        Value(unsafe { LLVMBuildFPToSI(self.0, value.0, ty.0, cstrp!("fp2si")) })
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

    pub fn build_br(&self, to: &BasicBlock) -> Value {
        Value(unsafe { LLVMBuildBr(self.0, to.0) })
    }

    pub fn build_condbr(&self, cond: &Value, then: &BasicBlock, el: &BasicBlock) -> Value {
        Value(unsafe { LLVMBuildCondBr(self.0, cond.0, then.0, el.0) })
    }
}
