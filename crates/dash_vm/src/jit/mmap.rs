use std::os::raw::c_void;
use std::ptr::{self, NonNull};

#[derive(Debug)]
pub struct MmapFn {
    ptr: NonNull<c_void>,
    len: usize,
}

impl MmapFn {
    pub fn call2<T1, T2, R>(&self, t1: T1, t2: T2) -> R {
        type RawMmapFn<T1, T2, R> = extern "C" fn(T1, T2) -> R;

        let f = unsafe { std::mem::transmute::<_, RawMmapFn<T1, T2, R>>(self.ptr.as_ptr()) };
        f(t1, t2)
    }

    pub fn call3<T1, T2, T3, R>(&self, t1: T1, t2: T2, t3: T3) -> R {
        type RawMmapFn<T1, T2, T3, R> = extern "C" fn(T1, T2, T3) -> R;

        let f = unsafe { std::mem::transmute::<_, RawMmapFn<T1, T2, T3, R>>(self.ptr.as_ptr()) };
        f(t1, t2, t3)
    }

    pub fn alloc(code: &[u8]) -> Self {
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
        Self { ptr, len: code.len() }
    }
}

impl Drop for MmapFn {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr.as_ptr(), self.len);
        }
    }
}
