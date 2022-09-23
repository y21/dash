use std::{cell::Cell, ptr::NonNull};

use crate::{gc::handle::InnerHandle, value::object::Object};

use self::{
    handle::{Handle, HandleFlags},
    linkedlist::{LinkedList, Node},
    trace::Trace,
};

pub mod handle;
pub mod linkedlist;
pub mod persistent;
pub mod trace;

pub struct Gc<T: ?Sized> {
    list: LinkedList<InnerHandle<T>>,
}

impl<T: ?Sized + Trace> Gc<T> {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    pub unsafe fn sweep(&mut self) {
        let mut previous = None;
        let mut node = self.list.tail();

        loop {
            if let Some(ptr) = node {
                let (flags, refcount, next) = {
                    let node = ptr.as_ref();

                    (&node.value.flags, node.value.refcount.get(), node.next)
                };

                node = next;

                if !flags.is_marked() && refcount == 0 {
                    // Reference did not get marked during GC trace and there are no Persistent<T> refs. Deallocate.

                    // If this node is the tail (i.e. oldest/first node) or there is no tail,
                    // set it to the next node.
                    let tail = self.list.tail_mut();
                    if tail.map_or(true, |p| p == ptr) {
                        *tail = next;
                    }

                    // If this node is the head (i.e. newest/last node) or there is no head,
                    // set it to the previous node.
                    let head = self.list.head_mut();
                    if head.map_or(true, |p| p == ptr) {
                        *head = previous;
                    }

                    // Finally, deallocate the node.
                    drop(Box::from_raw(ptr.as_ptr()));

                    // Update previous node's next ptr to the next pointer
                    if let Some(previous) = previous {
                        (*previous.as_ptr()).next = next;
                    }

                    // There's one less node now, so decrement length.
                    self.list.dec_len();
                } else {
                    flags.unmark();
                    previous = Some(ptr);
                }
            } else {
                break;
            }
        }
    }

    pub fn register<V>(&mut self, value: V) -> Handle<T>
    where
        V: IntoHandle<T, InnerHandle<T>>,
    {
        value.into_handle(&mut self.list)
    }
}

pub unsafe trait IntoHandle<T: ?Sized, U> {
    fn into_handle(self, link: &mut LinkedList<U>) -> Handle<T>;
}

unsafe impl<T: Object + 'static> IntoHandle<dyn Object, InnerHandle<dyn Object>> for T {
    fn into_handle(self, link: &mut LinkedList<InnerHandle<dyn Object>>) -> Handle<dyn Object> {
        let handle: InnerHandle<dyn Object> = InnerHandle {
            flags: HandleFlags::new(),
            value: Box::new(self),
            refcount: Cell::new(0),
        };

        let node = Node {
            next: None,
            value: handle,
        };

        unsafe {
            let ptr = NonNull::new_unchecked(link.add(Box::new(node)));
            Handle::new(ptr)
        }
    }
}
