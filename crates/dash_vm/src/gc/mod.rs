#![allow(unused)]

use bitflags::bitflags;
use std::borrow::Borrow;
use std::cell::Cell;
use std::ops::Deref;
use std::ptr::NonNull;

use crate::value::object::Object;

use self::handle::GcNode;
use self::handle::Handle;

pub mod handle;
pub mod persistent;
pub mod trace;

pub struct Gc {
    /// The very first node of this [`Gc`]
    head: Option<NonNull<GcNode<dyn Object>>>,
    /// The last-inserted node of this [`Gc`]
    tail: Option<NonNull<GcNode<dyn Object>>>,
    node_count: usize,
}

impl Default for Gc {
    fn default() -> Self {
        Self::new()
    }
}

impl Gc {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            node_count: 0,
        }
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }

    fn add(&mut self, value: Box<GcNode<dyn Object>>) -> Handle<dyn Object> {
        let ptr = NonNull::new(Box::into_raw(value)).unwrap();

        // insert head if this is the very first node
        if self.head.is_none() {
            self.head = Some(ptr);
        }

        // set the `next` pointer of the current last element in the list to this pointer
        if let Some(tail) = &mut self.tail {
            unsafe {
                tail.as_mut().next = Some(ptr);
            }
        }

        self.tail = Some(ptr);
        self.node_count += 1;

        unsafe { Handle::from_raw(ptr) }
    }

    pub fn register<O: Object + 'static>(&mut self, value: O) -> Handle<dyn Object> {
        value.into_handle(self)
    }

    /// # Safety
    /// Calling this function while there are unmarked, live [`Handle`]s is Undefined Behavior.
    /// Any unmarked node is deallocated during a sweep cycle.
    pub unsafe fn sweep(&mut self) {
        // The last valid pointer that was found
        let mut previous = None;
        let mut cur = self.head;

        while let Some(ptr) = cur {
            let GcNode {
                flags,
                refcount,
                next,
                ..
                // value,
            } = unsafe { ptr.as_ref() };

            cur = *next;

            if !flags.is_marked() && refcount.get() == 0 {
                // Reference did not get marked during mark phase
                // Deallocate and unlink!

                // If this node is the head (i.e. oldest/first node) or there is no head,
                // set it to the next node.
                if self.head.map_or(true, |p| p == ptr) {
                    self.head = *next;
                }

                // If this node is the tail (i.e. newest/most recently added node) or there is no tail,
                // set it to the last valid node.
                if self.tail.map_or(true, |p| p == ptr) {
                    self.tail = previous;
                }

                // Update last valid pointer to the next pointer
                if let Some(mut previous) = previous {
                    unsafe { previous.as_mut().next = *next };
                }

                // Deallocate node.
                unsafe { drop(Box::from_raw(ptr.as_ptr())) };

                // One less node now.
                self.node_count -= 1;
            } else {
                // Node still live
                flags.unmark();
                previous = Some(ptr);
            }
        }
    }
}

impl Drop for Gc {
    fn drop(&mut self) {
        let mut curr = self.head;
        while let Some(node) = curr {
            let next = unsafe { node.as_ref().next };
            curr = next;

            unsafe {
                drop(Box::from_raw(node.as_ptr()));
            }
        }
    }
}

macro_rules! register_gc {
    ($gc:expr, $val:expr) => {{
        let value = $val;
        let node = GcNode {
            flags: Default::default(),
            refcount: Default::default(),
            next: None,
            value,
        };
        $gc.add(Box::new(node))
    }};
}

/// # Safety
/// Implementors must provide a "correct" into_handle method
/// by returning a valid [`Handle`] living in the given linked list.
pub unsafe trait IntoHandle {
    fn into_handle(self, gc: &mut Gc) -> Handle<dyn Object>;
}

unsafe impl<T: Object + 'static> IntoHandle for T {
    fn into_handle(self, gc: &mut Gc) -> Handle<dyn Object> {
        register_gc!(gc, self)
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::any::TypeId;
    use std::fmt::Display;
    use std::rc::Rc;

    use crate::value::array::Array;
    use crate::value::object::NamedObject;
    use crate::value::ExternalValue;

    use super::*;

    #[test]
    fn gc_works() {
        unsafe {
            let mut gc = Gc::new();

            assert!(gc.node_count == 0);
            assert!(gc.head.is_none());
            assert!(gc.tail.is_none());

            let h1 = register_gc!(gc, 123.0);

            assert!(gc.head == NonNull::new(h1.as_ptr()));
            assert!(gc.tail == NonNull::new(h1.as_ptr()));
            assert!((*h1.as_ptr()).next.is_none());
            assert!(!(*h1.as_ptr()).flags.is_marked());
            assert!(gc.node_count == 1);

            let h2 = register_gc!(gc, Rc::from("hi"));

            assert!(gc.head == NonNull::new(h1.as_ptr()));
            assert!(gc.tail == NonNull::new(h2.as_ptr()));
            assert!((*h1.as_ptr()).next == NonNull::new(h2.as_ptr()));
            assert!(!(*h2.as_ptr()).flags.is_marked());
            assert!(gc.node_count == 2);

            (*h1.as_ptr()).flags.mark();
            (*h2.as_ptr()).flags.mark();

            assert!((*h1.as_ptr()).flags.is_marked());
            assert!((*h2.as_ptr()).flags.is_marked());

            gc.sweep();

            // nothing should have changed after GC sweep since all nodes were marked
            // they should be unmarked now though
            assert!(gc.head == NonNull::new(h1.as_ptr()));
            assert!(gc.tail == NonNull::new(h2.as_ptr()));
            assert!((*h1.as_ptr()).next == NonNull::new(h2.as_ptr()));
            assert!(!(*h1.as_ptr()).flags.is_marked());
            assert!(!(*h2.as_ptr()).flags.is_marked());
            assert!(gc.node_count == 2);

            // add a third node now
            let h3 = register_gc!(gc, true);

            assert!(gc.head == NonNull::new(h1.as_ptr()));
            assert!(gc.tail == NonNull::new(h3.as_ptr()));
            assert!((*h1.as_ptr()).next == NonNull::new(h2.as_ptr()));
            assert!((*h2.as_ptr()).next == NonNull::new(h3.as_ptr()));
            assert!(!(*h3.as_ptr()).flags.is_marked());
            assert!(gc.node_count == 3);

            // test handle casting
            {
                let h1_c = h1.cast_handle::<f64>();
                assert_eq!(h1_c.as_deref(), Some(&123.0));

                let h2_c = h2.cast_handle::<Rc<str>>();
                assert_eq!(h2_c.as_ref().map(|x| &***x), Some("hi"));

                let h3_c = h3.cast_handle::<bool>();
                assert_eq!(h3_c.as_deref(), Some(&true));

                // how about some invalid casts
                assert_eq!(h1.cast_handle::<bool>(), None);
                assert_eq!(h1.cast_handle::<Rc<str>>(), None);
                assert_eq!(h2.cast_handle::<bool>(), None);
                assert_eq!(h2.cast_handle::<Array>(), None);
                assert_eq!(h3.cast_handle::<f64>(), None);
                assert_eq!(h3.cast_handle::<NamedObject>(), None);
            }

            // ---

            // only mark second node
            (*h2.as_ptr()).flags.mark();

            gc.sweep();

            // only one node is left: h2
            assert!(gc.node_count == 1);
            assert!(gc.head == NonNull::new(h2.as_ptr()));
            assert!(gc.tail == NonNull::new(h2.as_ptr()));

            // final sweep
            gc.sweep();

            // nothing left.
            assert!(gc.node_count == 0);
            assert!(gc.head.is_none());
            assert!(gc.tail.is_none());

            // test that Handle::replace works
            {
                let h4i = register_gc!(gc, 123.0);
                let h4 = register_gc!(gc, ExternalValue::new(h4i));
                let mut h4c = h4.cast_handle::<ExternalValue>().unwrap();
                let h4i2 = register_gc!(gc, 456.0);
                ExternalValue::replace(&h4c, h4i2);
                let inner = h4c.inner.as_any().downcast_ref::<f64>().unwrap();
                assert_eq!(*inner, 456.0);
            }

            // lastly, test if Gc::drop works correctly. run under miri to see possible leaks
            register_gc!(gc, Rc::from("test"));
        }
    }
}
