use core::slice;
use std::alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc};
use std::ops::SubAssign;
use std::ptr::{self, NonNull};
use std::{cmp, iter, mem, ops};

use crate::util::unlikely;

/// # Safety
/// Implementors of this trait must provide valid members with the following invariants:
/// - `Self::ZERO` must be the zero value.
/// - `usize` must return the value as a `usize`. This is allowed to be a lossy conversion.
/// - `from_usize` must convert a `usize` to the type, which may be lossy.
pub unsafe trait IndexRepr: Sized {
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
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
    const MAX: Self = usize::MAX;
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
    const MAX: Self = u32::MAX;
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
    const MAX: Self = u16::MAX;
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
    const MAX: Self = u8::MAX;
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

#[derive(Debug)]
pub struct IndexVec<T, I: Index> {
    ptr: NonNull<T>,
    len: I::Repr,
    cap: I::Repr,
}

impl<T: Clone, I: Index> Clone for IndexVec<T, I> {
    fn clone(&self) -> Self {
        let mut new_vec = Self::with_capacity(self.cap);

        for value in self.as_slice().iter().cloned() {
            new_vec.push(value);
        }

        new_vec
    }
}

impl<T, I: Index> IndexVec<T, I> {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: I::Repr::default(),
            cap: I::Repr::default(),
        }
    }

    pub fn with_capacity(cap: I::Repr) -> Self {
        if cap == I::Repr::ZERO {
            return Self::new();
        }

        let layout = Layout::array::<T>(cap.usize()).unwrap();
        let ptr = unsafe { alloc(layout) };

        Self {
            ptr: NonNull::new(ptr)
                .unwrap_or_else(|| handle_alloc_error(layout))
                .cast::<T>(),
            len: I::Repr::default(),
            cap,
        }
    }

    pub fn repeat_n(element: T, count: I::Repr) -> Self
    where
        T: Clone,
    {
        let mut vec = Self::with_capacity(count);
        for value in iter::repeat_n(element, count.usize()) {
            vec.push(value);
        }
        vec
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
                let new_cap = cmp::min(new_cap, I::Repr::MAX.usize());
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
        self.try_push(value).expect("out of bounds repr")
    }

    pub fn try_push(&mut self, value: T) -> Option<I> {
        // The largest id we can give out is MAX - 1, since the length also shares the repr
        // and needs to be able to represent up to id + 1, which would overflow with MAX.
        if unlikely(self.len.usize() >= I::Repr::MAX.usize()) {
            return None;
        }

        self.ensure_capacity(1);

        let index = self.len;
        unsafe {
            ptr::write(self.ptr.as_ptr().add(index.usize()), value);
        }

        self.len = self.len.add(I::Repr::ONE);
        Some(I::from_repr(index))
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

    pub fn capacity(&self) -> I::Repr {
        self.cap
    }

    pub fn get(&self, index: I) -> Option<&T> {
        let index = index.into_repr();
        if index < self.len {
            Some(unsafe { &*self.ptr.as_ptr().add(index.usize()) })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
        let index = index.into_repr();
        if index < self.len {
            Some(unsafe { &mut *self.ptr.as_ptr().add(index.usize()) })
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

    pub fn shrink_to_fit(&mut self) {
        if self.cap == I::Repr::ZERO || self.len == I::Repr::ZERO || self.len == self.cap {
            return;
        }

        let old_cap = self.cap.usize();
        let new_cap = self.len.usize();
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        // Make a new allocation
        let ptr = unsafe { alloc(new_layout) };

        // Copy elements over
        unsafe { ptr::copy_nonoverlapping(self.ptr.as_ptr(), ptr.cast::<T>(), self.len().usize()) };

        // Deallocate old memory.
        unsafe { dealloc(self.ptr.as_ptr().cast::<u8>(), Layout::array::<T>(old_cap).unwrap()) };

        self.cap = I::Repr::from_usize(new_cap);
        self.ptr = NonNull::new(ptr)
            .unwrap_or_else(|| handle_alloc_error(new_layout))
            .cast::<T>();
    }
}

impl<T, I: Index> ops::Index<I> for IndexVec<T, I> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T, I: Index> ops::IndexMut<I> for IndexVec<T, I> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.get_mut(index).unwrap()
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

#[cfg(feature = "format")]
impl<T: serde::Serialize, I: Index> serde::Serialize for IndexVec<T, I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.len().usize()))?;
        for elem in self.iter() {
            seq.serialize_element(elem)?;
        }
        seq.end()
    }
}

#[cfg(feature = "format")]
impl<'de, T: serde::Deserialize<'de>, I: Index> serde::Deserialize<'de> for IndexVec<T, I> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::marker::PhantomData;

        struct Vis<T, I>(PhantomData<(T, I)>);
        impl<'de, T: serde::Deserialize<'de>, I: Index> serde::de::Visitor<'de> for Vis<T, I> {
            type Value = IndexVec<T, I>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut data = IndexVec::with_capacity(I::Repr::from_usize(seq.size_hint().unwrap_or_default()));
                while let Some(elem) = seq.next_element::<T>()? {
                    data.push(elem);
                }
                Ok(data)
            }
        }
        deserializer.deserialize_seq(Vis(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use crate::indexvec::IndexVec;

    index_type!(
        struct IdxU32(u32);
    );

    #[test]
    fn indexvec_ops() {
        let mut vec: IndexVec<String, IdxU32> = IndexVec::new();

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
        assert_eq!(vec.get(IdxU32(0)), Some(&"Item 0".to_string()));
        assert_eq!(vec.get(IdxU32(3)), None);

        for _ in 0..2000 {
            vec.push("New Item".to_string());
        }

        for i in (0..2003).rev() {
            vec.truncate(i);
        }
    }

    #[test]
    fn shrink_to_fit() {
        let mut vec: IndexVec<String, IdxU32> = IndexVec::new();
        for i in 0..100 {
            assert!(vec.capacity() >= vec.len());
            vec.push(format!("Item {}", i));
            assert!(vec.capacity() >= vec.len());
            vec.shrink_to_fit();
            assert_eq!(vec.len(), i + 1);
        }
    }

    #[test]
    fn indexvec_clone() {
        let mut vec: IndexVec<String, IdxU32> = IndexVec::new();
        for i in 0..10 {
            vec.push(format!("Item {}", i));
        }

        for _ in 0..10 {
            let cloned_vec = vec.clone();
            assert_eq!(cloned_vec.len(), 10);
            for i in 0..10 {
                assert_eq!(cloned_vec.get(IdxU32(i)), Some(&format!("Item {}", i)));
            }
        }
    }

    #[test]
    fn repeat_n() {
        let vec: IndexVec<String, IdxU32> = IndexVec::repeat_n("Repeated Item".to_string(), 5);
        assert_eq!(vec.len(), 5);
        for i in 0..5 {
            assert_eq!(vec.get(IdxU32(i)), Some(&"Repeated Item".to_string()));
        }
    }

    #[test]
    #[should_panic = "out of bounds repr"]
    fn handles_overflow() {
        index_type!(
            struct Idx(u8);
        );
        let mut vec: IndexVec<String, Idx> = IndexVec::new();

        for i in 0..255 {
            vec.push(format!("Item {}", i));
        }
        vec.push("Item 255".to_string()); // This should panic
    }

    #[test]
    fn handles_overflow_with_try_push() {
        index_type!(
            #[derive(Debug, Clone, Copy, Eq, PartialEq)]
            struct Idx(u8);
        );
        let mut vec: IndexVec<String, Idx> = IndexVec::new();

        for i in 0..255 {
            assert_eq!(vec.try_push(format!("Item {}", i)), Some(Idx(i as u8)));
            assert_eq!(vec[Idx(i as u8)], format!("Item {}", i));
        }
        assert!(vec.try_push("Item 255".to_string()).is_none());
        assert_eq!(vec.len(), 255);
    }
}
