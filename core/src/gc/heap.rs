use std::marker::PhantomData;

/// A heap allocated heap node
pub struct Node<T> {
    /// The value of this node
    pub value: T,
    /// A pointer to the next node in the heap
    pub next: Option<*mut Node<T>>,
}

impl<T> From<T> for Node<T> {
    fn from(value: T) -> Self {
        Self { value, next: None }
    }
}

/// A datastructure similar to a linked list which allows efficiently removing nodes
#[derive(Clone, Debug)]
pub struct Heap<T> {
    /// Top of the heap (value that was added last)
    pub head: Option<*mut Node<T>>,
    /// Bottom of the heap (value that was added first)
    pub tail: Option<*mut Node<T>>,
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
    pub fn add<N: Into<Node<T>>>(&mut self, value: N) -> *mut T {
        let node = Box::into_raw(Box::new(value.into()));

        if let Some(head) = self.head {
            unsafe {
                (*head).next = Some(node);
            }
        }

        if self.tail.is_none() {
            self.tail = Some(node);
        }

        self.head = Some(node);
        self.len += 1;

        unsafe { &mut (*node).value as *mut T }
    }

    /// Returns an iterator over the heap
    pub fn iter(&self) -> HeapIter<'_, T> {
        HeapIter {
            next: self.tail,
            _heap: PhantomData,
        }
    }
}

impl<T> Drop for Heap<T> {
    fn drop(&mut self) {
        let mut next = self.tail;

        while let Some(ptr) = next {
            unsafe {
                next = (*ptr).next;
                Box::from_raw(ptr)
            };
        }
    }
}

/// An iterator over the values in a heap
pub struct HeapIter<'a, T> {
    /// The next node to be returned
    next: Option<*mut Node<T>>,
    /// PhantomData to ensure that HeapIter does not outlive its Heap
    _heap: PhantomData<&'a ()>,
}

impl<'a, T: 'a> Iterator for HeapIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        if let Some(ptr) = next {
            unsafe {
                self.next = (*ptr).next;
                Some(&(*ptr).value)
            }
        } else {
            None
        }
    }
}
