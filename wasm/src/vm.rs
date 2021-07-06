use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    rc::Rc,
};

use dash::{
    compiler::compiler::Compiler,
    parser::{lexer::Lexer, parser::Parser},
    vm::{
        value::{
            function::{Constructor, FunctionType, UserFunction},
            Value,
        },
        VM,
    },
};

use crate::{
    error::{CreateVMError, VMInterpretError},
    ffi::{WasmOption, WasmResult},
    handle::{Handle, HandleRef},
};

macro_rules! try_result {
    ($e:expr) => {
        match $e {
            Ok(o) => o,
            Err(e) => return Handle::new(WasmResult::Err(e.into())),
        }
    };
}

#[no_mangle]
pub extern "C" fn eval<'a>(
    source: *const i8,
) -> Handle<WasmResult<WasmOption<Rc<RefCell<Value>>>, CreateVMError<'a>>> {
    let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
    let result = WasmOption::from(try_result!(dash::eval::<()>(source, None)));
    Handle::new(WasmResult::Ok(result))
}

#[no_mangle]
pub extern "C" fn create_vm<'a>(source: *const i8) -> Handle<WasmResult<VM, CreateVMError<'a>>> {
    let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };

    let tokens = try_result!(Lexer::new(source).scan_all());
    let stmts = try_result!(Parser::new(source, tokens).parse_all());
    let bytecode = try_result!(Compiler::<()>::new(stmts, None, false).compile());
    let func = UserFunction::new(bytecode, 0, FunctionType::Top, 0, Constructor::NoCtor);

    Handle::new(WasmResult::Ok(VM::new(func)))
}

#[no_mangle]
pub extern "C" fn vm_interpret(
    mut vm: HandleRef<VM>,
) -> Handle<WasmResult<WasmOption<Rc<RefCell<Value>>>, VMInterpretError>> {
    let vm = unsafe { vm.as_mut() };
    let value = WasmOption::from(try_result!(vm.interpret()));
    Handle::new(WasmResult::Ok(value))
}

#[no_mangle]
pub extern "C" fn value_inspect(value: HandleRef<Rc<RefCell<Value>>>) -> *mut i8 {
    let value_cell = unsafe { value.as_ref() };
    let value = value_cell.borrow();
    let string = CString::new(&*value.inspect(0)).unwrap();
    string.into_raw()
}

#[no_mangle]
pub extern "C" fn value_to_string(value: HandleRef<Rc<RefCell<Value>>>) -> *mut i8 {
    let value_cell = unsafe { value.as_ref() };
    let value = value_cell.borrow();
    let string = CString::new(&*value.to_string()).unwrap();
    string.into_raw()
}

#[no_mangle]
pub extern "C" fn free_create_vm_result<'a>(value: Handle<WasmResult<VM, CreateVMError<'a>>>) {
    unsafe { value.drop() };
}

#[no_mangle]
pub extern "C" fn free_eval_result<'a>(
    value: Handle<WasmResult<WasmOption<Rc<RefCell<Value>>>, CreateVMError<'a>>>,
) {
    unsafe { value.drop() };
}

#[no_mangle]
pub extern "C" fn free_vm_interpret_result<'a>(
    value: Handle<WasmResult<WasmOption<Rc<RefCell<Value>>>, VMInterpretError>>,
) {
    unsafe { value.drop() };
}
