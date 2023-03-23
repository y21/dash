use std::iter::Enumerate;
use std::slice;
use std::slice::Iter;

use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::instruction::IntrinsicOperation;

#[macro_export]
macro_rules! cstrp {
    ($string:expr) => {
        cstr::cstr!($string).as_ptr()
    };
}

/// # Safety
/// See [`slice::from_raw_parts_mut`]
pub unsafe fn transmute_slice_mut<T, U>(slice: &mut [T]) -> &mut [U] {
    slice::from_raw_parts_mut(slice.as_mut_ptr().cast::<U>(), slice.len())
}
