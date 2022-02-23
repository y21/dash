use std::fmt::Debug;

use super::{
    handle::InnerHandleGuard,
    heap::{Heap, Node},
    Handle,
};

/// A tracing garbage collector
#[derive(Debug)]
pub struct Gc<T> {
    /// The underlying heap
    pub heap: Heap<InnerHandleGuard<T>>,
}

impl<T> Gc<T> {
    /// Creates a new garbage collector
    pub fn new() -> Self {
        Self { heap: Heap::new() }
    }

    /// Performs a GC cycle
    ///
    /// It scans through the heap and deallocates every object that is not marked as visited.
    /// This operation is unsafe as it invalidates any [Handle] that may be still alive.
    pub unsafe fn sweep(&mut self) {
        let mut previous = None;
        let mut node = self.heap.tail;

        loop {
            if let Some(mut ptr) = node {
                let (marked, next) = unsafe {
                    let node = ptr.as_ref();

                    (node.value.get_unchecked().is_marked(), node.next)
                };

                node = next;

                if !marked {
                    // No more references to the object, we can deallocate it

                    // If this is the heap tail, we need to update it
                    if self.heap.tail.map(|p| p == ptr).unwrap_or(true) {
                        self.heap.tail = next;
                    }

                    // If this is the heap head, we need to update it
                    if self.heap.head.map(|p| p == ptr).unwrap_or(true) {
                        self.heap.head = previous;
                    }

                    // Finally, deallocate
                    drop(unsafe { Box::from_raw(ptr.as_ptr()) });

                    // Update previous node's next ptr to the next pointer
                    if let Some(mut previous) = previous {
                        unsafe {
                            previous.as_mut().next = next;
                        };
                    }

                    self.heap.len -= 1;
                } else {
                    unsafe { ptr.as_mut().value.get_mut_unchecked().unmark_visited() };
                    previous = Some(ptr);
                }
            } else {
                break;
            }
        }
    }

    /// Registers a new value
    ///
    /// If not marked as visited, the returned [Handle] will dangle when sweep is called.
    pub fn register<H>(&mut self, value: H) -> Handle<T>
    where
        H: Into<InnerHandleGuard<T>> + Debug,
    {
        let ptr = self.heap.add(value.into());

        // SAFETY: the pointer is valid
        unsafe { Handle::new(ptr) }
    }

    /// Transfers all values from the provided [Gc<T>] to self
    pub fn transfer(&mut self, gc: Gc<T>) {
        let mut heap = gc.heap;

        if self.heap.len == 0 {
            // if our heap is empty, we can cheat by just swapping instead of appending
            self.heap = heap;
            return;
        }

        // slow path: append all objects
        let mut next = heap.tail;

        while let Some(ptr) = next {
            unsafe {
                next = ptr.as_ref().next;
                if let Some(mut head) = self.heap.head {
                    head.as_mut().next = Some(ptr);
                }

                self.heap.head = Some(ptr);
                self.heap.len += 1;
            }
        }

        // set old heap tail/head to None so that its destructor doesn't deallocate moved objects
        heap.tail = None;
        heap.head = None;
    }
}
