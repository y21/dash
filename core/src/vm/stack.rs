use std::{
    fmt::Debug,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct OwnedStack<T, const N: usize>(Stack<T, N>);
impl<T, const N: usize> OwnedStack<T, N> {
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

#[derive(Debug)]
pub struct Stack<T, const N: usize>([MaybeUninit<T>; N], usize);

impl<T, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        unsafe { Self(MaybeUninit::uninit().assume_init(), 0) }
    }

    pub fn into_iter(self, order: IteratorOrder) -> StackIterator<T, N> {
        let top = self.1;
        StackIterator {
            array: self.into_array(),
            index: 0,
            order,
            top,
        }
    }

    pub fn into_array(self) -> [MaybeUninit<T>; N] {
        self.0
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
        self.get_unchecked_at(self.1 - 1)
    }

    pub unsafe fn get_unchecked_at(&self, at: usize) -> &T {
        &*self.0.get_unchecked(at).as_ptr()
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

    pub fn reset_stack_pointer(&mut self) {
        self.1 = 0;
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

    pub fn reset(&mut self) {
        for mu in self.0.iter_mut().take(self.1) {
            let mu = std::mem::replace(mu, MaybeUninit::uninit());

            drop(unsafe { mu.assume_init() });
        }

        self.reset_stack_pointer();
    }
}

#[derive(Copy, Clone)]
pub enum IteratorOrder {
    TopToBottom, // 9 --> 0
    BottomToTop, // 0 --> 9
}

// TODO: implement Drop and free remaining elements ?
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
                if self.index <= 0 {
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
