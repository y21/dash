use llvm_sys::core::LLVMConstInt;
use llvm_sys::core::LLVMDoubleType;
use llvm_sys::core::LLVMFloatType;
use llvm_sys::core::LLVMInt1Type;
use llvm_sys::core::LLVMInt64Type;
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::prelude::LLVMValueRef;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum Value {
    Integer(i64),
    Number(f64),
    Boolean(bool),
}

impl Value {
    pub const SIZE_OF_LARGEST: usize = std::mem::size_of::<i64>() * 8;

    pub fn to_const_value(self) -> LLVMValueRef {
        unsafe {
            let ty = self.type_of();
            match self {
                Value::Integer(i) => LLVMConstInt(ty, i as u64, 0),
                Value::Number(f) => LLVMConstInt(ty, f.to_bits(), 0),
                Value::Boolean(b) => LLVMConstInt(ty, b as u64, 0),
            }
        }
    }

    pub fn type_of(self) -> LLVMTypeRef {
        unsafe {
            match self {
                Value::Integer(_) => LLVMInt64Type(),
                Value::Boolean(_) => LLVMInt1Type(),
                Value::Number(_) => LLVMDoubleType()
            }
        }
    }
}

#[cfg(test)]
#[test]
fn value_mem_layout() {
    todo!("Write this test to ensure memory layout. Everything. Including SIZE_OF_LARGEst")
}
