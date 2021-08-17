use super::{
    handle::InnerHandleGuard,
    heap::{Heap, Node},
    Handle,
};

// todo: static AtomicUsize for next id instead of using pointers?
/// A marker that uniquely identifies an instance of a type
#[derive(Clone, Debug)]
pub struct Marker(Box<u8>);

impl Marker {
    pub(crate) fn new() -> Self {
        Self(Box::new(0))
    }

    /// Returns the marker pointer
    pub fn get(&self) -> *const () {
        &*self.0 as *const u8 as *const ()
    }
}

/// A tracing garbage collector
#[derive(Debug)]
pub struct Gc<T> {
    /// The underlying heap
    pub heap: Heap<InnerHandleGuard<T>>,
    /// A unique marker for this specific GC
    pub marker: Marker,
}

impl<T> Gc<T> {
    /// Creates a new garbage collector
    pub fn new() -> Self {
        Self {
            heap: Heap::new(),
            marker: Marker::new(),
        }
    }

    /// Performs a GC cycle
    ///
    /// It scans through the heap and deallocates every object that is not marked as visited.
    /// This operation is unsafe as it invalidates any [Handle] that may be still alive.
    pub unsafe fn sweep(&mut self) {
        let mut previous = <Option<*mut Node<InnerHandleGuard<T>>>>::None;
        let mut node = self.heap.tail;

        loop {
            if let Some(ptr) = node {
                let (marked, next) = unsafe {
                    let node = &*ptr;

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
                    drop(unsafe { Box::from_raw(ptr) });

                    // Update previous node's next ptr to the next pointer
                    if let Some(previous) = previous {
                        unsafe {
                            (*previous).next = next;
                        };
                    }

                    self.heap.len -= 1;
                } else {
                    unsafe { (*ptr).value.get_mut_unchecked().unmark_visited() };
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
        H: Into<InnerHandleGuard<T>>,
    {
        let ptr = self.heap.add(value.into());
        unsafe { Handle::new(ptr, self.marker.get()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::value::{Value, ValueKind};
    use std::ptr::eq as ptr_eq;

    fn assert_node_eq<T>(l: Option<*mut Node<T>>, r: *mut T) {
        assert!(l
            .map(|x| unsafe { ptr_eq(&(*x).value, r) })
            .unwrap_or(false));
    }

    #[test]
    pub fn gc_single_value_reclaim() {
        let mut gc = Gc::new();
        let handle = gc.register(Value::new(ValueKind::Number(123f64)));
        let ptr = handle.as_ptr();

        // When gc only has one element, len must be 1
        assert_eq!(gc.heap.len, 1);

        // Assert that when gc only has one element, tail and head must point to it
        assert_node_eq(gc.heap.head, ptr);
        assert_node_eq(gc.heap.tail, ptr);

        unsafe { gc.sweep() };

        // After GC sweep, len must be 0
        assert_eq!(gc.heap.len, 0);

        // Tail and head must be none
        assert!(gc.heap.tail.is_none());
        assert!(gc.heap.head.is_none());
    }

    #[test]
    pub fn gc_multi_value_reclaim() {
        let mut gc = Gc::new();

        let handle1 = gc.register(Value::new(ValueKind::Number(123f64)));
        let handle2 = gc.register(Value::new(ValueKind::Number(456f64)));
        let ptr1 = handle1.as_ptr();
        let ptr2 = handle2.as_ptr();

        assert_eq!(gc.heap.len, 2);

        // Tail = 1st element
        assert_node_eq(gc.heap.tail, ptr1);
        // Head = last element
        assert_node_eq(gc.heap.head, ptr2);

        unsafe { gc.sweep() };

        assert_eq!(gc.heap.len, 0);
        assert!(gc.heap.tail.is_none());
        assert!(gc.heap.head.is_none());
    }

    #[test]
    pub fn gc_single_value_mark() {
        let mut gc = Gc::new();

        let handle1 = gc.register(Value::new(ValueKind::Number(123f64)));
        unsafe { handle1.borrow_mut_unbounded() }.mark_visited();
        let ptr1 = handle1.as_ptr();

        assert_eq!(gc.heap.len, 1);

        unsafe { gc.sweep() };

        assert_eq!(gc.heap.len, 1);

        assert_node_eq(gc.heap.tail, ptr1);
        assert_node_eq(gc.heap.head, ptr1);

        // cleanup
        unsafe { gc.sweep() };
    }

    #[test]
    pub fn gc_multi_value_mark() {
        let mut gc = Gc::new();

        let handle1 = gc.register(Value::new(ValueKind::Number(123f64)));
        let handle2 = gc.register(Value::new(ValueKind::Number(456f64)));
        unsafe { handle1.borrow_mut_unbounded() }.mark_visited();
        unsafe { handle2.borrow_mut_unbounded() }.mark_visited();
        let ptr1 = handle1.as_ptr();
        let ptr2 = handle2.as_ptr();

        assert_eq!(gc.heap.len, 2);

        unsafe { gc.sweep() };

        assert_eq!(gc.heap.len, 2);

        assert_node_eq(gc.heap.tail, ptr1);
        assert_node_eq(gc.heap.head, ptr2);

        // cleanup
        unsafe { gc.sweep() };
    }

    // We are relying on the fact that *const () does not get optimized to a pointer to 0x1 like Box::new(()) does it
    // (Box::new is defined to not allocate on ZSTs, but we add a test case anyway)
    #[test]
    pub fn const_ptr_unit() {
        let x = Box::new(0u8);
        let y = &*x as *const u8;
        let z = &*x as *const _ as *const ();
        assert!(y as usize == z as usize);
    }
}
