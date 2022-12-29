#![allow(unused)]

use llvm_sys::execution_engine::LLVMLinkInMCJIT;
use llvm_sys::target::LLVM_InitializeNativeAsmParser;
use llvm_sys::target::LLVM_InitializeNativeAsmPrinter;
use llvm_sys::target::LLVM_InitializeNativeTarget;

pub mod backend;
pub mod function;
pub mod legacy;
pub mod passes;
pub mod error;

pub use backend::Backend;
pub use legacy::assembler::Assembler;
pub use legacy::trace::Trace;
pub use legacy::value::Value;

pub fn init() {
    unsafe {
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();
        LLVM_InitializeNativeAsmParser();
        LLVMLinkInMCJIT();
    }
}
