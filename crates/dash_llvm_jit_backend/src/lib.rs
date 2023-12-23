#![allow(unused)]

use llvm_sys::execution_engine::LLVMLinkInMCJIT;
use llvm_sys::target::{LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget};

pub mod codegen;
pub mod error;
mod llvm_wrapper;
pub mod trace;
pub mod util;

pub use trace::Trace;

pub fn init() {
    unsafe {
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();
        LLVM_InitializeNativeAsmParser();
        LLVMLinkInMCJIT();
    }
}
