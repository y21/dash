#![allow(unused)]

use llvm_sys::execution_engine::LLVMLinkInMCJIT;
use llvm_sys::target::LLVM_InitializeNativeAsmParser;
use llvm_sys::target::LLVM_InitializeNativeAsmPrinter;
use llvm_sys::target::LLVM_InitializeNativeTarget;

pub mod codegen;
pub mod error;
mod llvm_wrapper;
pub mod passes;
pub mod trace;
pub mod typed_cfg;
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
