use std::{alloc::Layout, ffi::CString};

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    unsafe { std::alloc::alloc(Layout::from_size_align(len, 1).unwrap()) }
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    unsafe { std::alloc::dealloc(ptr, Layout::from_size_align(len, 1).unwrap()) };
}

#[no_mangle]
pub extern "C" fn free_c_string(ptr: *mut i8) {
    unsafe { CString::from_raw(ptr) };
}
