use core::slice;
use std::alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc};
use std::ops::SubAssign;
use std::ptr::{self, NonNull};
use std::{cmp, mem, ops};

use crate::util::unlikely;

/// # Safety
/// Implementors of this trait must provide valid members with the following invariants:
/// - `Self::ZERO` must be the zero value.
/// - `usize` must return the value as a `usize`. This is allowed to be a lossy conversion.
/// - `from_usize` must convert a `usize` to the type, which may be lossy.
pub unsafe trait IndexRepr: Sized {
    const ZERO: Self;
    const ONE: Self;
    fn usize(self) -> usize;
    fn from_usize(n: usize) -> Self {
        Self::from_usize_checked(n).expect("out of bounds repr")
    }
    unsafe fn from_usize_unchecked(n: usize) -> Self;
    fn from_usize_checked(n: usize) -> Option<Self>;

    fn add(self, other: Self) -> Self;
    fn mul(self, other: Self) -> Self;
}

unsafe impl IndexRepr for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    fn usize(self) -> usize {
        self
    }
    fn from_usize_checked(n: usize) -> Option<Self> {
        Some(n)
    }
    unsafe fn from_usize_unchecked(n: usize) -> Self {
        n
    }
    fn add(self, other: Self) -> Self {
        self.checked_add(other).unwrap()
    }
    fn mul(self, other: Self) -> Self {
        self.checked_mul(other).unwrap()
    }
}

unsafe impl IndexRepr for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    fn usize(self) -> usize {
        self as usize
    }
    fn from_usize_checked(n: usize) -> Option<Self> {
        u32::try_from(n).ok()
    }
    unsafe fn from_usize_unchecked(n: usize) -> Self {
        n as u32
    }
    fn add(self, other: Self) -> Self {
        self.checked_add(other).unwrap()
    }
    fn mul(self, other: Self) -> Self {
        self.checked_mul(other).unwrap()
    }
}

unsafe impl IndexRepr for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    fn usize(self) -> usize {
        self as usize
    }
    fn from_usize_checked(n: usize) -> Option<Self> {
        u16::try_from(n).ok()
    }
    unsafe fn from_usize_unchecked(n: usize) -> Self {
        n as u16
    }
    fn add(self, other: Self) -> Self {
        self.checked_add(other).unwrap()
    }
    fn mul(self, other: Self) -> Self {
        self.checked_mul(other).unwrap()
    }
}

unsafe impl IndexRepr for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    fn usize(self) -> usize {
        self as usize
    }
    fn from_usize_checked(n: usize) -> Option<Self> {
        u8::try_from(n).ok()
    }
    unsafe fn from_usize_unchecked(n: usize) -> Self {
        n as u8
    }
    fn add(self, other: Self) -> Self {
        self.checked_add(other).unwrap()
    }
    fn mul(self, other: Self) -> Self {
        self.checked_mul(other).unwrap()
    }
}

pub trait Index {
    type Repr: Default + Copy + IndexRepr + SubAssign + Eq + Ord;
    fn from_repr(repr: Self::Repr) -> Self;
    fn into_repr(self) -> Self::Repr;
}

pub struct IndexVec<T, I: Index> {
    ptr: NonNull<T>,
    len: I::Repr,
    cap: I::Repr,
}

impl<T, I: Index> IndexVec<T, I> {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: I::Repr::default(),
            cap: I::Repr::default(),
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len.usize()) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len.usize()) }
    }

    pub fn pop(&mut self) -> Option<T> {
        let len = self.len.usize();

        if len == 0 {
            None
        } else {
            self.len -= I::Repr::ONE;
            let value = unsafe { ptr::read(self.ptr.add(len - 1).as_ptr()) };
            Some(value)
        }
    }

    fn ensure_capacity(&mut self, additional: usize) {
        let new_len = self.len.add(I::Repr::from_usize(additional)).usize();

        if unlikely(new_len > self.cap.usize()) {
            if self.cap == I::Repr::ZERO {
                // Initial allocation.
                let cap = cmp::max(additional, 4);
                let layout = Layout::array::<T>(cap).unwrap();

                self.ptr = NonNull::new(unsafe { alloc(layout) })
                    .unwrap_or_else(|| handle_alloc_error(layout))
                    .cast::<T>();
                self.cap = I::Repr::from_usize(cap);
            } else {
                // Reallocation.
                let current_layout = Layout::array::<T>(self.cap.usize()).unwrap();
                let new_cap = cmp::max(self.cap.usize() * 2, self.len.usize() + additional);
                let new_size = Layout::array::<T>(new_cap).unwrap().size();
                // NB: from_usize asserts that there is no overflow, so this must happen before performing the realloc
                let new_cap = I::Repr::from_usize(new_cap);

                self.ptr = NonNull::new(unsafe { realloc(self.ptr.as_ptr().cast::<u8>(), current_layout, new_size) })
                    .unwrap_or_else(|| handle_alloc_error(current_layout))
                    .cast::<T>();
                self.cap = new_cap;
            }
        }
    }

    pub fn push(&mut self, value: T) -> I {
        self.ensure_capacity(1);

        let index = self.len;
        unsafe {
            ptr::write(self.ptr.as_ptr().add(index.usize()), value);
        }

        self.len = self.len.add(I::Repr::ONE);
        I::from_repr(index)
    }

    pub fn last(&self) -> Option<&T> {
        if let [.., last] = self.as_slice() {
            Some(last)
        } else {
            None
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if let [.., last] = self.as_mut_slice() {
            Some(last)
        } else {
            None
        }
    }

    pub fn len(&self) -> I::Repr {
        self.len
    }

    pub fn get(&self, index: I) -> Option<&T> {
        let index = index.into_repr();
        if index < self.len {
            Some(unsafe { &*self.ptr.as_ptr().add(index.usize()) })
        } else {
            None
        }
    }

    pub fn truncate(&mut self, new_len: I::Repr) {
        let new_len = new_len.usize();
        if new_len < self.len.usize() {
            if mem::needs_drop::<T>() {
                let remaining = self.len.usize() - new_len;
                // SAFETY: new_len is in bounds
                let slice_to_drop = unsafe { ptr::slice_from_raw_parts_mut(self.ptr.add(new_len).as_ptr(), remaining) };

                // SAFETY: all elements are initialized
                unsafe { ptr::drop_in_place(slice_to_drop) };
            }

            // SAFETY: new_len is less than the current length, so it is a valid repr value
            self.len = unsafe { I::Repr::from_usize_unchecked(new_len) };
        }
    }
}

impl<T, I: Index> ops::Index<I> for IndexVec<T, I> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T, I: Index> ops::Deref for IndexVec<T, I> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, I: Index> ops::DerefMut for IndexVec<T, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, I: Index> Drop for IndexVec<T, I> {
    fn drop(&mut self) {
        if self.cap != I::Repr::ZERO {
            if mem::needs_drop::<T>() {
                // SAFETY: all elements are initialized
                unsafe { ptr::drop_in_place(slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len.usize())) };
            }

            // SAFETY: we allocated the pointer with alloc/dealloc and the same layout
            unsafe {
                dealloc(
                    self.ptr.as_ptr().cast::<u8>(),
                    Layout::array::<T>(self.cap.usize()).unwrap(),
                );
            }
        }
    }
}

#[macro_export]
macro_rules! index_type {
    (
        $(
            $(#[$meta:meta])*
            $vis:vis struct $name:ident($fvis:vis $repr:ty);
        )*
    ) => {
        $(
            $(#[$meta])*
            $vis struct $name($fvis $repr);

            impl $crate::indexvec::Index for $name {
                type Repr = $repr;
                fn from_repr(repr: Self::Repr) -> Self {
                    Self(repr)
                }
                fn into_repr(self) -> Self::Repr {
                    self.0
                }
            }
        )*
    };
}

#[cfg(test)]
mod tests {
    use crate::indexvec::IndexVec;

    #[test]
    fn indexvec_ops() {
        index_type!(
            struct Idx(u32);
        );
        let mut vec: IndexVec<String, Idx> = IndexVec::new();

        let fill = |vec: &mut IndexVec<_, _>| {
            for i in 0..20 {
                vec.push(format!("Item {}", i));
            }
        };
        assert!(vec.pop().is_none());
        assert_eq!(vec.len(), 0);
        fill(&mut vec);
        assert_eq!(vec.len(), 20);

        for i in (0..20).rev() {
            assert_eq!(vec.pop(), Some(format!("Item {}", i)));
        }
        assert!(vec.pop().is_none());
        assert_eq!(vec.len(), 0);
        fill(&mut vec);
        assert_eq!(vec.len(), 20);
        vec.truncate(2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(Idx(0)), Some(&"Item 0".to_string()));
        assert_eq!(vec.get(Idx(3)), None);

        for _ in 0..2000 {
            vec.push("New Item".to_string());
        }

        for i in (0..2003).rev() {
            vec.truncate(i);
        }
    }

    #[test]
    #[should_panic = "out of bounds repr"]
    fn handles_overflow() {
        index_type!(
            struct Idx(u8);
        );
        let mut vec: IndexVec<String, Idx> = IndexVec::new();

        for i in 0..257 {
            vec.push(format!("Item {}", i));
        }
    }
}
