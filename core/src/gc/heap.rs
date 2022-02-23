use std::{marker::PhantomData, ptr::NonNull};

/// A heap allocated heap node
pub struct Node<T: ?Sized> {
    /// A pointer to the next node in the heap
    pub next: Option<NonNull<Node<T>>>,
    /// The value of this node
    pub value: T,
}

impl<T> From<T> for Node<T> {
    fn from(value: T) -> Self {
        Self { value, next: None }
    }
}

/// A datastructure similar to a linked list which allows efficiently removing nodes
#[derive(Clone, Debug)]
pub struct Heap<T: ?Sized> {
    /// Top of the heap (value that was added last)
    pub head: Option<NonNull<Node<T>>>,
    /// Bottom of the heap (value that was added first)
    pub tail: Option<NonNull<Node<T>>>,
    /// The length of the heap
    pub len: usize,
}

impl<T> Heap<T> {
    /// Creates a new heap
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    /// Adds a new value to this heap
    pub fn add(&mut self, value: T) -> *mut T {
        let node = Box::into_raw(Box::new(Node::from(value)));
        // SAFETY: the pointer returned by Box::into_raw is guaranteed to be non null
        let node = unsafe { NonNull::new_unchecked(node) };

        if let Some(mut head) = self.head {
            unsafe {
                // SAFETY: if self.head is some, then the contained pointer is valid
                head.as_mut().next = Some(node);
            }
        }

        if self.tail.is_none() {
            self.tail = Some(node);
        }

        self.head = Some(node);
        self.len += 1;

        unsafe { (&mut (*node.as_ptr()).value) as *mut T }
    }

    /// Returns an iterator over the heap
    pub fn iter(&self) -> HeapIter<'_, T> {
        HeapIter {
            next: self.tail,
            _heap: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for Heap<T> {
    fn drop(&mut self) {
        let mut next = self.tail;

        while let Some(ptr) = next {
            let ptr = ptr.as_ptr();

            unsafe {
                next = (*ptr).next;
                Box::from_raw(ptr);
            };
        }
    }
}

/// An iterator over the values in a heap
pub struct HeapIter<'a, T> {
    /// The next node to be returned
    next: Option<NonNull<Node<T>>>,
    /// PhantomData to ensure that HeapIter does not outlive its Heap
    _heap: PhantomData<&'a ()>,
}

impl<'a, T: 'a> Iterator for HeapIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        if let Some(ptr) = next {
            let ptr = ptr.as_ptr();

            unsafe {
                self.next = (*ptr).next;
                Some(&(*ptr).value)
            }
        } else {
            None
        }
    }
}
