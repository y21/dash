#![allow(unused)]

use llvm_sys::execution_engine::LLVMLinkInMCJIT;
use llvm_sys::target::LLVM_InitializeNativeAsmParser;
use llvm_sys::target::LLVM_InitializeNativeAsmPrinter;
use llvm_sys::target::LLVM_InitializeNativeTarget;

pub mod trace;
pub mod assembler;

pub fn init() {
    unsafe {
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();
        LLVM_InitializeNativeAsmParser();
        LLVMLinkInMCJIT();
    }
}
