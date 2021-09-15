use std::ffi::{CStr, CString};

use dash::{
    agent::Agent,
    gc::Handle as GcHandle,
    vm::{value::Value, VMError, VM},
};

use crate::{
    error::CreateVMError,
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

#[repr(C)]
pub struct Eval {
    value: WasmOption<Handle<GcHandle<Value>>>,
    vm: Handle<VM>,
}

type EvalResult<'a> = WasmResult<Eval, CreateVMError<'a>>;
type CreateVMFromStringResult<'a> = WasmResult<Handle<VM>, CreateVMError<'a>>;
type InterpretVMResult = WasmResult<WasmOption<Handle<GcHandle<Value>>>, VMError>;
type VMEvalResult<'a> = WasmResult<WasmOption<Handle<GcHandle<Value>>>, CreateVMError<'a>>;

fn create_agent() -> impl Agent {
    runtime::agent(runtime::agent_flags::FS | runtime::agent_flags::MEM)
}

#[no_mangle]
pub extern "C" fn eval<'a>(source: *const i8) -> Handle<EvalResult<'a>> {
    let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
    let (value, vm) = try_result!(dash::eval(source, Some(create_agent())));
    let value = WasmOption::from(value.map(Handle::new));
    let vm = Handle::new(vm);
    Handle::new(WasmResult::Ok(Eval { value, vm }))
}

#[no_mangle]
pub extern "C" fn create_vm() -> Handle<VM> {
    Handle::new(VM::new())
}

#[no_mangle]
pub extern "C" fn create_vm_from_string<'a>(
    source: *const i8,
) -> Handle<CreateVMFromStringResult<'a>> {
    let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
    let vm = try_result!(VM::from_str(source, Some(create_agent())));
    Handle::new(WasmResult::Ok(Handle::new(vm)))
}

#[no_mangle]
pub extern "C" fn vm_interpret(mut vm: HandleRef<VM>) -> Handle<InterpretVMResult> {
    let vm = unsafe { vm.as_mut() };
    let value = try_result!(vm.interpret());
    Handle::new(WasmResult::Ok(WasmOption::from(value.map(Handle::new))))
}

#[no_mangle]
pub extern "C" fn vm_eval<'a>(
    mut vm: HandleRef<VM>,
    source: *const i8,
) -> Handle<VMEvalResult<'a>> {
    let vm = unsafe { vm.as_mut() };
    let source = unsafe { CStr::from_ptr(source).to_str().unwrap() };
    let value = try_result!(vm.eval(source));
    Handle::new(WasmResult::Ok(WasmOption::from(value.map(Handle::new))))
}

#[no_mangle]
pub extern "C" fn value_inspect(value: HandleRef<GcHandle<Value>>) -> *mut i8 {
    let value = unsafe { value.as_ref() };
    let value_ref = unsafe { value.borrow_unbounded() };
    let inspected = value_ref.inspect(0);
    let string = CString::new(&*inspected).unwrap();
    string.into_raw()
}

// TODO: this should return... a result?
// it may call a user function, which can throw an error
#[no_mangle]
pub extern "C" fn value_to_string(value: HandleRef<GcHandle<Value>>) -> *mut i8 {
    let value = unsafe { value.as_ref() };
    let value_ref = unsafe { value.borrow_unbounded() };
    let inspected = value_ref.to_string();
    let string = CString::new(&*inspected).unwrap();
    string.into_raw()
}

#[no_mangle]
pub extern "C" fn vm_set_gc_object_threshold(mut vm: HandleRef<VM>, threshold: usize) {
    let vm = unsafe { vm.as_mut() };
    vm.set_gc_object_threshold(threshold);
}

#[no_mangle]
pub extern "C" fn vm_run_async_tasks(mut vm: HandleRef<VM>) {
    let vm = unsafe { vm.as_mut() };
    vm.run_async_tasks();
}

macro_rules! define_destructors {
    ($($name:ident => $type:ty),*) => {
        $(
            #[no_mangle]
            pub extern "C" fn $name(handle: Handle<$type>) {
                unsafe { handle.drop() };
            }
        )*
    }
}

define_destructors! {
    free_vm => VM,
    free_eval_result => EvalResult,
    free_create_vm_from_string_result => CreateVMFromStringResult,
    free_vm_interpret_result => InterpretVMResult,
    free_vm_eval => VMEvalResult
}
