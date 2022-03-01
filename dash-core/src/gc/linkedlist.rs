use std::ptr::{addr_of_mut, NonNull};

/// A node in the linked list
pub struct Node<T: ?Sized> {
    pub next: Option<NonNull<Node<T>>>,
    pub value: T,
}

/// An implementation of a linked list that's actually useful
/// as opposed to `std::collections::LinkedList`...
pub struct LinkedList<T: ?Sized> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

impl<T: ?Sized> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn add(&mut self, value: Box<Node<T>>) -> *mut T {
        let ptr = Box::into_raw(value);
        let nptr = unsafe { NonNull::new_unchecked(ptr) };

        // if we have a head, set the next pointer
        if let Some(head) = &mut self.head {
            unsafe {
                head.as_mut().next = Some(nptr);
            }
        }

        // if this is the first node (that is, tail is None), set the tail
        if self.tail.is_none() {
            self.tail = Some(nptr);
        }

        // update this list's head
        self.head = Some(nptr);
        self.len += 1;

        unsafe { addr_of_mut!((*ptr).value) }
    }
}

impl<T: ?Sized> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut curr = self.tail;
        while let Some(node) = curr {
            let next = unsafe { node.as_ref().next };
            curr = next;

            unsafe {
                drop(Box::from_raw(node.as_ptr()));
            }
        }
    }
}
