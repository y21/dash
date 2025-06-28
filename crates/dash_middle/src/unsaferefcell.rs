#[cfg(not(debug_assertions))]
use std::cell::UnsafeCell;
#[cfg(debug_assertions)]
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};

/// Semantically, this is equivalent to an `UnsafeCell<T>`,
/// i.e. it's unsafe to borrow and must be proven by the caller
/// to be safe.
///
/// In debug builds it panics if the value is already borrowed,
/// but this is only for extra sanity checks.
#[derive(Debug)]
pub struct UnsafeRefCell<T> {
    #[cfg(debug_assertions)]
    cell: RefCell<T>,
    #[cfg(not(debug_assertions))]
    cell: UnsafeCell<T>,
}

impl<T> UnsafeRefCell<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            #[cfg(debug_assertions)]
            cell: std::cell::RefCell::new(value),
            #[cfg(not(debug_assertions))]
            cell: UnsafeCell::new(value),
        }
    }

    /// # Safety
    /// The caller must ensure that no other `UnsafeRefMut`s or `UnsafeRef`s coexist.
    #[inline]
    pub unsafe fn borrow_mut(&self) -> UnsafeRefMut<'_, T> {
        #[cfg(debug_assertions)]
        {
            UnsafeRefMut(self.cell.borrow_mut())
        }
        #[cfg(not(debug_assertions))]
        {
            // SAFETY: The caller must ensure that no other (mutable) references exist.
            UnsafeRefMut(unsafe { &mut *self.cell.get() })
        }
    }

    /// # Safety
    /// The caller must ensure that no other `UnsafeRefMut`s coexist.
    pub unsafe fn borrow(&self) -> UnsafeRef<'_, T> {
        #[cfg(debug_assertions)]
        {
            UnsafeRef(self.cell.borrow())
        }
        #[cfg(not(debug_assertions))]
        {
            // SAFETY: The caller must ensure that no mutable references exist.
            UnsafeRef(unsafe { &*self.cell.get() })
        }
    }
}

#[repr(transparent)]
pub struct UnsafeRef<'a, T>(#[cfg(debug_assertions)] Ref<'a, T>, #[cfg(not(debug_assertions))] &'a T);

impl<T> Deref for UnsafeRef<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[repr(transparent)]
pub struct UnsafeRefMut<'a, T>(
    #[cfg(debug_assertions)] RefMut<'a, T>,
    #[cfg(not(debug_assertions))] &'a mut T,
);

impl<T> Deref for UnsafeRefMut<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> DerefMut for UnsafeRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
