use std::convert::Infallible;
use std::marker::PhantomData;

use crate::compiler::constant::ConstantPool;
use crate::iterator_with::IteratorWith;

pub trait ExtractSource: Sized {
    type Value: ExtractBack<Self, Exception = Infallible> + ExtractFront<Self, Exception = Infallible>;
    type Unrooted: ExtractBack<Self, Exception = Infallible>;

    fn fetch_bytes<const N: usize>(&mut self) -> [u8; N];

    fn constants(&self) -> &ConstantPool;

    fn stack_len(&self) -> usize;

    fn pop_stack_rooted(&mut self) -> Self::Value;

    fn pop_stack(&mut self) -> Self::Unrooted;

    fn peek_stack_rooted(&mut self) -> Self::Value;

    fn peek_stack(&self) -> Self::Unrooted;

    fn truncate_stack(&mut self, len: usize);

    fn fetch_u32(&mut self) -> u32 {
        let bytes = self.fetch_bytes::<4>();
        u32::from_le_bytes(bytes)
    }

    fn fetch_u16(&mut self) -> u16 {
        let bytes = self.fetch_bytes::<2>();
        u16::from_le_bytes(bytes)
    }

    fn fetch_u8(&mut self) -> u8 {
        let bytes = self.fetch_bytes::<1>();
        bytes[0]
    }
}

pub trait ExtractBack<S: ExtractSource>: Sized {
    type Exception;

    fn extract_back(source: &mut S) -> Result<Self, Self::Exception>;
}

pub fn extract_back_infallible<S: ExtractSource, T: ExtractBack<S, Exception = Infallible>>(source: &mut S) -> T {
    let Ok(value) = T::extract_back(source);
    value
}

pub fn extract_back<S: ExtractSource, T: ExtractBack<S>>(source: &mut S) -> Result<T, T::Exception> {
    T::extract_back(source)
}

pub fn extract_front_infallible<S: ExtractSource, T: ExtractFront<S, Exception = Infallible>, U>(
    source: &mut S,
    seq: &mut ForwardSequence<U>,
) -> T {
    let Ok(value) = T::extract_front(source, seq);
    value
}

pub fn extract_front<S: ExtractSource, T: ExtractFront<S>, U>(
    source: &mut S,
    seq: &mut ForwardSequence<U>,
) -> Result<T, T::Exception> {
    T::extract_front(source, seq)
}

pub trait ExtractFront<S: ExtractSource>: Sized {
    type Exception;

    fn extract_front<U>(cx: &mut S, seq: &mut ForwardSequence<U>) -> Result<Self, Self::Exception>;
}

/// A lazy sequence of values that takes an index at which values start and yields them forwards from there.
pub struct ForwardSequence<T> {
    /// How many more values can we yield from `ForwardSequence::next`?
    remaining_len: usize,
    /// The current stack index, incremented when extracting from the front.
    stack_index: usize,
    /// The stack index at which values start (used to reset the stack to at the end), fixed at creation
    starting_stack_index: usize,
    _phantom: PhantomData<T>,
}

impl<T> ForwardSequence<T> {
    pub fn from_stack_count_len(source: &impl ExtractSource, count: usize, len: usize) -> Self {
        let stack_index = source.stack_len() - count;
        Self {
            remaining_len: len,
            stack_index,
            starting_stack_index: stack_index,
            _phantom: PhantomData,
        }
    }

    pub fn next_stack_index(&mut self) -> usize {
        let index = self.stack_index;
        self.stack_index += 1;
        index
    }

    pub fn commit(self, source: &mut impl ExtractSource) {
        source.truncate_stack(self.starting_stack_index);
    }
}

pub struct BackwardSequence<T> {
    /// How many more values can we yield from `BackwardSequence::next`?
    remaining_len: usize,
    _phantom: PhantomData<T>,
}

impl<T> BackwardSequence<T> {
    pub fn new_u16(source: &mut impl ExtractSource) -> Self {
        let len = source.fetch_u16();
        Self {
            remaining_len: len.into(),
            _phantom: PhantomData,
        }
    }
}

impl<S: ExtractSource, T: ExtractBack<S>> IteratorWith<&mut S> for BackwardSequence<T> {
    type Item = Result<T, T::Exception>;

    fn next(&mut self, args: &mut S) -> Option<Self::Item> {
        if self.remaining_len > 0 {
            self.remaining_len -= 1;
            Some(extract_back(args))
        } else {
            None
        }
    }
}

impl<S: ExtractSource, T: ExtractBack<S>> ExtractBack<S> for BackwardSequence<T> {
    type Exception = Infallible;

    fn extract_back(source: &mut S) -> Result<Self, Self::Exception> {
        Ok(Self::new_u16(source))
    }
}

impl<S: ExtractSource, T: ExtractFront<S>> IteratorWith<&mut S> for ForwardSequence<T> {
    type Item = Result<T, T::Exception>;

    fn next(&mut self, args: &mut S) -> Option<Self::Item> {
        if self.remaining_len > 0 {
            self.remaining_len -= 1;
            Some(extract_front(args, self))
        } else {
            None
        }
    }
}
