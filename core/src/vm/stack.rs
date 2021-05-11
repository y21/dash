use std::ops::Drop;
use std::{fmt::Debug, mem::MaybeUninit};

#[derive(Debug)]
pub struct Stack<T, const N: usize>([MaybeUninit<T>; N], usize);

impl<T, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        unsafe { Self(MaybeUninit::uninit().assume_init(), 0) }
    }

    pub fn len(&self) -> usize {
        self.1
    }

    pub fn push(&mut self, v: T) {
        assert!(N > self.1);

        unsafe { self.0[self.1].as_mut_ptr().write(v) };
        self.1 += 1;
    }

    pub fn pop(&mut self) -> T {
        assert!(self.1 > 0);

        self.1 -= 1;
        let old = &mut self.0[self.1];
        let val = std::mem::replace(old, MaybeUninit::uninit());
        unsafe { val.assume_init() }
    }

    pub unsafe fn get_unchecked(&self) -> &T {
        &*self.0[self.1 - 1].as_ptr()
    }

    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        &mut *self.0[self.1 - 1].as_mut_ptr()
    }

    pub fn set(&mut self, idx: usize, value: T) {
        self.set_relative(0, idx, value)
    }

    pub fn set_relative(&mut self, offset: usize, idx: usize, value: T) {
        assert!(offset + idx <= self.1);

        if offset + idx == self.1 {
            self.1 += 1;
        }

        unsafe { self.0[offset + idx].as_mut_ptr().write(value) }
    }

    pub fn get_stack_pointer(&self) -> usize {
        self.1
    }

    pub fn set_stack_pointer(&mut self, sp: usize) {
        self.1 = sp;
    }

    pub unsafe fn peek_unchecked(&self, idx: usize) -> &T {
        self.peek_relative_unchecked(0, idx)
    }

    pub unsafe fn peek_relative_unchecked(&self, offset: usize, idx: usize) -> &T {
        assert!(offset + idx < self.1);

        &*self.0[offset + idx].as_ptr()
    }

    pub fn reset(&mut self) {
        self.1 = 0;
    }

    /// Discards all values on the stack except for the one at the top
    pub fn into_last(&mut self) {
        let last = self.pop();

        for _ in 0..self.1 {
            self.pop();
        }

        self.push(last);
    }

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
}

impl<T, const N: usize> Drop for Stack<T, N> {
    fn drop(&mut self) {
        for mu in self.0.iter_mut().take(self.1) {
            let mu = std::mem::replace(mu, MaybeUninit::uninit());
            drop(unsafe { mu.assume_init() })
        }
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
