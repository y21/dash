use std::ffi::CString;

use dash::{
    compiler::compiler::CompileError,
    gc::handle::Handle as GcHandle,
    parser::{lexer::Error as LexError, token::Error as ParseError},
    vm::{value::Value, FromStrError, VMError, VM},
    EvalError,
};

use crate::handle::{Handle, HandleRef};

#[derive(Debug)]
#[repr(C)]
pub enum CreateVMErrorKind<'a> {
    Lexer(Vec<LexError<'a>>),
    Parser(Vec<ParseError<'a>>),
    Compiler(CompileError<'a>),
    VM(VMError),
}

#[derive(Debug)]
#[repr(C)]
pub struct CreateVMError<'a> {
    pub kind: CreateVMErrorKind<'a>,
    pub vm: Option<Handle<VM>>,
}

impl<'a> CreateVMError<'a> {
    pub fn new(kind: CreateVMErrorKind<'a>) -> Self {
        Self { kind, vm: None }
    }

    pub fn with_vm(kind: CreateVMErrorKind<'a>, vm: Option<VM>) -> Self {
        Self {
            kind,
            vm: vm.map(Handle::new),
        }
    }
}

impl<'a> From<Vec<LexError<'a>>> for CreateVMError<'a> {
    fn from(value: Vec<LexError<'a>>) -> Self {
        Self::new(CreateVMErrorKind::Lexer(value))
    }
}

impl<'a> From<Vec<ParseError<'a>>> for CreateVMError<'a> {
    fn from(value: Vec<ParseError<'a>>) -> Self {
        Self::new(CreateVMErrorKind::Parser(value))
    }
}

impl<'a> From<CompileError<'a>> for CreateVMError<'a> {
    fn from(value: CompileError<'a>) -> Self {
        Self::new(CreateVMErrorKind::Compiler(value))
    }
}

impl<'a> From<VMError> for CreateVMError<'a> {
    fn from(value: VMError) -> Self {
        Self::new(CreateVMErrorKind::VM(value))
    }
}

impl<'a> From<(EvalError<'a>, Option<VM>)> for CreateVMError<'a> {
    fn from(value: (EvalError<'a>, Option<VM>)) -> Self {
        match value.0 {
            EvalError::LexError(l) => Self::with_vm(CreateVMErrorKind::Lexer(l), value.1),
            EvalError::ParseError(p) => Self::with_vm(CreateVMErrorKind::Parser(p), value.1),
            EvalError::CompileError(c) => Self::with_vm(CreateVMErrorKind::Compiler(c), value.1),
            EvalError::VMError(v) => Self::with_vm(CreateVMErrorKind::VM(v), value.1),
        }
    }
}

impl<'a> From<EvalError<'a>> for CreateVMError<'a> {
    fn from(value: EvalError<'a>) -> Self {
        match value {
            EvalError::LexError(l) => Self::new(CreateVMErrorKind::Lexer(l)),
            EvalError::ParseError(p) => Self::new(CreateVMErrorKind::Parser(p)),
            EvalError::CompileError(c) => Self::new(CreateVMErrorKind::Compiler(c)),
            EvalError::VMError(v) => Self::new(CreateVMErrorKind::VM(v)),
        }
    }
}

impl<'a> From<FromStrError<'a>> for CreateVMError<'a> {
    fn from(value: FromStrError<'a>) -> Self {
        match value {
            FromStrError::CompileError(c) => Self::new(CreateVMErrorKind::Compiler(c)),
            FromStrError::LexError(l) => Self::new(CreateVMErrorKind::Lexer(l)),
            FromStrError::ParseError(p) => Self::new(CreateVMErrorKind::Parser(p)),
        }
    }
}

#[no_mangle]
pub extern "C" fn inspect_create_vm_error(e: HandleRef<CreateVMError<'_>>) -> *mut i8 {
    let e = unsafe { e.as_ref() };
    let msg = match &e.kind {
        CreateVMErrorKind::Lexer(l) => l
            .iter()
            .map(|e| e.to_string().to_string())
            .collect::<Vec<String>>()
            .join("\n"),
        CreateVMErrorKind::Parser(p) => p
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("\n"),
        CreateVMErrorKind::Compiler(c) => c.to_string().to_string(),
        CreateVMErrorKind::VM(v) => v.to_string().to_string(),
    };

    CString::new(msg).unwrap().into_raw()
}

#[repr(C)]
pub enum VMInterpretError {
    UncaughtError(GcHandle<Value>),
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
