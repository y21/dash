use std::{cell::RefCell, ffi::CString, rc::Rc};

use dash::{
    compiler::compiler::CompileError,
    parser::{lexer::Error as LexError, token::Error as ParseError},
    vm::{value::Value, VMError},
};

use crate::handle::{Handle, HandleRef};

#[derive(Debug)]
#[repr(C)]
pub enum CreateVMError<'a> {
    Lexer(Vec<LexError<'a>>),
    Parser(Vec<ParseError<'a>>),
    Compiler(CompileError<'a>),
}

impl<'a> From<Vec<LexError<'a>>> for CreateVMError<'a> {
    fn from(value: Vec<LexError<'a>>) -> Self {
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

#[no_mangle]
pub extern "C" fn inspect_create_vm_error(e: HandleRef<CreateVMError<'_>>) -> *mut i8 {
    let e = unsafe { e.as_ref() };
    let msg = match e {
        CreateVMError::Lexer(l) => l
            .iter()
            .map(|e| e.to_string().to_string())
            .collect::<Vec<String>>()
            .join("\n"),
        CreateVMError::Parser(p) => p
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("\n"),
        CreateVMError::Compiler(c) => c.to_string().to_string(),
    };

    CString::new(msg).unwrap().into_raw()
}

#[repr(C)]
#[derive(Debug)]
pub enum VMInterpretError {
    UncaughtError(Rc<RefCell<Value>>),
}

impl From<VMError> for VMInterpretError {
    fn from(value: VMError) -> Self {
        match value {
            VMError::UncaughtError(e) => Self::UncaughtError(e),
        }
    }
}

impl From<VMInterpretError> for VMError {
    fn from(value: VMInterpretError) -> VMError {
        match value {
            VMInterpretError::UncaughtError(e) => VMError::UncaughtError(e),
        }
    }
}

#[no_mangle]
pub extern "C" fn inspect_vm_interpret_error(err: Handle<VMInterpretError>) -> *mut i8 {
    let err = unsafe { *err.into_box() };
    let err = VMError::from(err);
    let err = CString::new(&*err.to_string()).unwrap();
    err.into_raw()
}
