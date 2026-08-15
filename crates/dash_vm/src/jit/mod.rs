use std::ffi::c_void;
use std::mem;
use std::ptr::{self, NonNull};

use crate::frame::Ip;
use crate::localscope::LocalScope;

mod x86;

pub struct MmapFn {
    ptr: NonNull<c_void>,
    len: usize,
}

impl MmapFn {
    unsafe fn call0<R>(&self) -> R {
        let f = unsafe { mem::transmute::<_, unsafe extern "C" fn() -> R>(self.ptr.as_ptr()) };
        f()
    }
}

impl Drop for MmapFn {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr.as_ptr(), self.len);
        }
    }
}

fn mmap_jit_fn(code: &[u8]) -> MmapFn {
    let ptr = NonNull::new(unsafe {
        libc::mmap(
            ptr::null_mut(),
            code.len(),
            libc::PROT_READ | libc::PROT_EXEC | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    })
    .expect("failed to mmap jit region");

    unsafe { ptr.copy_from_nonoverlapping(NonNull::new(code.as_ptr().cast_mut()).unwrap().cast(), code.len()) };
    MmapFn { ptr, len: code.len() }
}

#[test]
fn testing_x86() {
    let mut x86 = x86::Emitter::new();
    x86.mov_reg_imm32(x86::Register::Eax, 42);
    x86.ret();

    let fn_ptr = mmap_jit_fn(x86.buffer());
    dbg!(unsafe { fn_ptr.call0::<u32>() });
}

pub fn compile_loop_region(scope: &mut LocalScope<'_>, start: Ip, end: Ip) {
    scope.frames.with_current_bytecode(|bytecode| {
        let loop_bytecode = &bytecode[start.0 as usize..end.0 as usize];

        let mut x86 = x86::Emitter::new();
        x86.mov_reg_imm32(x86::Register::Eax, 42);

        println!("Bytecode = {loop_bytecode:?}");
    });
}
