use std::{
    fmt::Debug,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use crate::gc::Handle;

use super::{frame::Frame, value::Value};

/// An owned stack
///
/// It wraps [Stack] and implements the [Drop] trait,
/// where its contents are deallocated
#[derive(Debug)]
pub struct OwnedStack<T, const N: usize>(Stack<T, N>);
impl<T, const N: usize> OwnedStack<T, N> {
    /// Creates a new owned stack
    pub fn new() -> Self {
        Self(Stack::new())
    }
}

impl<T, const N: usize> Deref for OwnedStack<T, N> {
    type Target = Stack<T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for OwnedStack<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const N: usize> Drop for OwnedStack<T, N> {
    fn drop(&mut self) {
        self.0.reset();
    }
}

/// A stack data structure
///
/// It has a fixed capacity and lives on the stack.
#[derive(Debug)]
pub struct Stack<T, const N: usize>([MaybeUninit<T>; N], usize);

impl<const N: usize> Stack<Handle<Value>, N> {
    /// Marks every handle in the stack as visited
    pub fn mark_visited(&self) {
        for handle in self.as_array() {
            let handle = unsafe { &*handle.as_ptr() };
            Value::mark(handle)
        }
    }
}

impl<const N: usize> Stack<Frame, N> {
    /// Marks every frame as visited
    pub fn mark_visited(&self) {
        for frame in self.as_array() {
            let frame = unsafe { &*frame.as_ptr() };
            frame.mark_visited();
        }
    }
}

impl<T, const N: usize> Stack<T, N> {
    /// Creates a new stack
    pub fn new() -> Self {
        unsafe { Self(MaybeUninit::uninit().assume_init(), 0) }
    }

    /// Returns an iterator for this stack
    pub fn into_iter(self, order: IteratorOrder) -> StackIterator<T, N> {
        let top = self.1;
        StackIterator {
            array: self.into_array(),
            index: 0,
            order,
            top,
        }
    }

    /// Returns the underlying array for this stack
    pub fn into_array(self) -> [MaybeUninit<T>; N] {
        self.0
    }

    /// Returns a bool, indicating whether this stack is empty
    pub fn is_empty(&self) -> bool {
        self.1 == 0
    }

    /// Stack length (number of filled items)
    pub fn len(&self) -> usize {
        self.1
    }

    /// Returns an iterator over the initialized items
    pub fn as_array(&self) -> impl Iterator<Item = &MaybeUninit<T>> {
        self.0.iter().take(self.1)
    }

    /// Returns an iterator over the initialized items, starting at the top
    pub fn as_array_bottom(&self) -> impl Iterator<Item = &MaybeUninit<T>> {
        self.0.iter().take(self.1).rev()
    }

    /// Pushes a value on the stack
    pub fn push(&mut self, v: T) {
        assert!(N > self.1);

        unsafe { self.0[self.1].as_mut_ptr().write(v) };
        self.1 += 1;
    }

    /// Pops the last value that was pushed off the stack and returns it
    pub fn pop(&mut self) -> T {
        assert!(self.1 > 0);

        self.1 -= 1;
        let old = &mut self.0[self.1];
        let val = std::mem::replace(old, MaybeUninit::uninit());
        unsafe { val.assume_init() }
    }

    /// Pops multiple values off the stack
    pub fn pop_multiple(&mut self, count: usize) -> Vec<T> {
        let mut v = Vec::with_capacity(self.len());
        for _ in 0..count {
            v.push(self.pop());
        }
        v
    }

    /// Pops multiple values off the stack and discards them
    pub fn discard_multiple(&mut self, count: usize) {
        for _ in 0..count {
            self.pop();
        }
    }

    /// Returns a reference to the last value
    pub fn get(&self) -> Option<&T> {
        if self.1 > 0 && N >= self.1 {
            // SAFETY: The index is checked against the length of the array
            Some(unsafe { &*self.0[self.1 - 1].as_ptr() })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the last value
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.1 > 0 && N >= self.1 {
            // SAFETY: The index is checked against the length of the array
            Some(unsafe { &mut *self.0[self.1 - 1].as_mut_ptr() })
        } else {
            None
        }
    }

    /// Returns a reference to the last value, assuming it is initialized
    ///
    /// No boundary checks are performed
    pub unsafe fn get_unchecked(&self) -> &T {
        self.get_unchecked_at(self.1 - 1)
    }

    /// Returns a reference to value at any position, assuming it is initialized
    ///
    /// No boundary checks are performed
    pub unsafe fn get_unchecked_at(&self, at: usize) -> &T {
        &*self.0.get_unchecked(at).as_ptr()
    }

    /// Returns a mutable reference to the last value, assuming it is initialized
    ///
    /// No boundary checks are performed
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        &mut *self.0[self.1 - 1].as_mut_ptr()
    }

    /// Sets a value at a given position
    pub fn set(&mut self, idx: usize, value: T) {
        self.set_relative(0, idx, value)
    }

    /// Sets a value at a given position, relative from `offset`
    pub fn set_relative(&mut self, offset: usize, idx: usize, value: T) {
        assert!(offset + idx <= self.1);

        if offset + idx == self.1 {
            self.1 += 1;
        }

        unsafe { self.0[offset + idx].as_mut_ptr().write(value) }
    }

    /// Sets the stack pointer
    pub unsafe fn set_stack_pointer(&mut self, sp: usize) {
        self.1 = sp;
    }

    /// Returns a reference to the current value, assuming it is initialized
    pub unsafe fn peek_unchecked(&self, idx: usize) -> &T {
        self.peek_relative_unchecked(0, idx)
    }

    /// Returns a reference to a value relative from `offset`, assuming it is initialized
    pub unsafe fn peek_relative_unchecked(&self, offset: usize, idx: usize) -> &T {
        assert!(offset + idx < self.1);

        &*self.0[offset + idx].as_ptr()
    }

    /// Resets the stack pointer
    pub fn reset_stack_pointer(&mut self) {
        self.1 = 0;
    }

    /// Iterates over the stack, applies the predicate on each value
    /// and returns the element that caused the function to return true
    pub fn find<F>(&self, f: F) -> Option<(usize, &T)>
    where
        F: Fn(&T) -> bool,
    {
        for (idx, value) in self.0.iter().take(self.1).enumerate() {
            let value = unsafe { &*value.as_ptr() };
            if f(value) {
                return Some((idx, value));
            }
        }

        None
    }

    /// Dumps the stack
    ///
    /// This iterates over the array and prints the value of each item
    pub fn dump(&self)
    where
        T: Debug,
    {
        println!("=== STACK DUMP [sp={}] ===", self.1);
        for (idx, val) in self.0.iter().take(self.1).enumerate() {
            let val = unsafe { &*val.as_ptr() };
            println!("{:04}    {:?}", idx, val);
        }
    }

    /// Replaces the underlying array with an empty array and returns the old stack
    pub fn take(&mut self) -> Stack<T, N> {
        let stack: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let old_stack = std::mem::replace(&mut self.0, stack);
        let old_index = self.1;
        self.1 = 0;

        Stack(old_stack, old_index)
    }

    /// Resets this stack by dropping every value and setting the stack pointer to 0
    pub fn reset(&mut self) {
        for mu in self.0.iter_mut().take(self.1) {
            let mu = std::mem::replace(mu, MaybeUninit::uninit());

            drop(unsafe { mu.assume_init() });
        }

        self.reset_stack_pointer();
    }
}

/// The order of a stack iterator
#[derive(Copy, Clone)]
pub enum IteratorOrder {
    /// Starting at the top
    TopToBottom, // 9 --> 0
    /// Starting at the bottom
    BottomToTop, // 0 --> 9
}

/// An owned iterator over a stack
// TODO: implement Drop and free remaining elements?
pub struct StackIterator<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    order: IteratorOrder,
    index: usize,
    top: usize,
}

impl<T, const N: usize> Iterator for StackIterator<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = match self.order {
            IteratorOrder::BottomToTop => {
                if self.index >= self.top {
                    return None;
                }
                self.index += 1;
                self.index - 1
            }
            IteratorOrder::TopToBottom => {
                if self.index == 0 {
                    return None;
                }
                self.index -= 1;
                self.index + 1
            }
        };

        let value = std::mem::replace(&mut self.array[idx], MaybeUninit::uninit());

        unsafe { Some(value.assume_init()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn stack() {
        let mut s: Stack<_, 127> = Stack::new();

        for i in 0..127 {
            s.push(i);
        }

        for i in (0..127).rev() {
            assert_eq!(s.pop(), i);
        }
    }
}
