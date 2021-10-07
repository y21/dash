use std::ffi::CString;

#[no_mangle]
pub fn version() -> *mut i8 {
    let version = CString::new(dash::VERSION).unwrap();
    version.into_raw()
}
