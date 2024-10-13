use std::alloc::{handle_alloc_error, Layout};
use std::mem::{self, ManuallyDrop};
use std::ptr::NonNull;
use std::{alloc, cmp};

/// Essentially Vec<u8>, but the allocation is aligned to 8 bytes (but not any of the inner bytes).
/// Also does not expose any of the bytes directly as there can be padding/uninit bytes
pub struct AlignedBuf {
    ptr: NonNull<u8>,
    cap: usize,
    len: usize,
}

impl AlignedBuf {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }

    /// Only call this if growing is necessary. If you just want to make space that might already be available, call reserve()
    fn grow_amortized(&mut self, additional: usize) {
        // Partially adapted from RawVec with unnecessary things removed that only matter for generic code
        debug_assert!(additional > 0);

        let new_cap = cmp::max(self.cap * 2, self.len + additional);
        let ptr = if self.cap == 0 {
            // We haven't allocated yet.
            unsafe { alloc::alloc(Layout::from_size_align(new_cap, 8).unwrap()) }
        } else {
            let old_layout = Layout::from_size_align(self.cap, 8).unwrap();
            unsafe { alloc::realloc(self.ptr.as_ptr(), old_layout, new_cap) }
        };

        self.ptr =
            NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(Layout::from_size_align(new_cap, 8).unwrap()));
        self.cap = new_cap;
    }

    pub fn reserve(&mut self, bytes: usize) {
        if self.len + bytes > self.cap {
            self.grow_amortized(bytes);
        }
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, byte: u8) {
        if self.len == self.cap {
            self.grow_amortized(1);
        }
        // SAFETY: the returned pointer is in bounds and valid for 1-byte writes, because we've reserved space for it
        unsafe { self.insertion_point().write(byte) };
        self.len += 1;
    }

    pub fn push_n(&mut self, byte: u8, count: usize) {
        for _ in 0..count {
            self.push(byte);
        }
    }

    /// Returns a pointer at which elements can be inserted
    ///
    /// # Safety
    /// Calling this function requires that the capacity is greater than the length (that is, there must be space available)
    pub unsafe fn insertion_point(&self) -> *mut u8 {
        debug_assert!(self.cap > self.len);
        self.as_ptr().add(self.len)
    }

    /// Copies a `T` into the buffer, not running any destructors
    ///
    /// # Safety
    /// Requires that the T is aligned when written at the current position
    pub unsafe fn write<T>(&mut self, value: T) {
        const {
            assert!(size_of::<T>() > 0);
        }

        self.reserve(size_of::<T>());
        unsafe { self.insertion_point().cast::<T>().write(value) }
        self.len += size_of::<T>();
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        if self.cap > 0 {
            // SAFETY: the pointer was previously allocated in grow_amorized
            unsafe { alloc::dealloc(self.ptr.as_ptr(), Layout::from_size_align(self.cap, 8).unwrap()) };
        }
    }
}
