
use std::{cell::RefCell, ffi::CString, rc::Rc};

use dash::{compiler::compiler::CompileError, parser::{
    lexer::Error as LexError,
    token::Error as ParseError
}, vm::{VMError, value::Value}};

use crate::handle::HandleRef;

#[derive(Debug)]
#[repr(C)]
pub enum CreateVMError<'a> {
    Lexer(Vec<LexError>),
    Parser(Vec<ParseError<'a>>),
    Compiler(CompileError<'a>),
    VM(VMError)
}

impl<'a> From<Vec<LexError>> for CreateVMError<'a> {
    fn from(value: Vec<LexError>) -> Self {
        Self::Lexer(value)
    }
}

impl<'a> From<Vec<ParseError<'a>>> for CreateVMError<'a> {
    fn from(value: Vec<ParseError<'a>>) -> Self {
        Self::Parser(value)
    }
}

impl<'a> From<CompileError<'a>> for CreateVMError<'a> {
    fn from(value: CompileError<'a>) -> Self {
        Self::Compiler(value)
    }
}

impl<'a> From<VMError> for CreateVMError<'a> {
    fn from(value: VMError) -> Self {
        Self::VM(value)
    }
}

#[no_mangle]
pub extern "C" fn inspect_create_vm_error(e: HandleRef<CreateVMError<'_>>) -> *mut i8 {
    let e = unsafe { e.as_ref() };
    CString::new(format!("{:?}", e)).unwrap().into_raw()
}

#[repr(C)]
#[derive(Debug)]
pub enum VMInterpretError {
    UncaughtError(Rc<RefCell<Value>>)
}

impl From<VMError> for VMInterpretError {
    fn from(value: VMError) -> Self {
        match value {
            VMError::UncaughtError(e) => Self::UncaughtError(e)
        }
    }
}

#[no_mangle]
pub extern "C" fn inspect_vm_interpret_error(e: HandleRef<VMError>) -> *mut i8 {
    let e = unsafe { e.as_ref() };
    CString::new(format!("{:?}", e)).unwrap().into_raw()
}