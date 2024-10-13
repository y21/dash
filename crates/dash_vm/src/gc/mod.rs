use std::fmt::Debug;
use std::ptr::NonNull;

use dash_log::debug;
use handle::ObjectVTable;

use crate::value::object::Object;

use self::handle::{GcNode, Handle};

mod buf;
pub mod gc2;
pub mod handle;
pub mod interner;
pub mod persistent;
pub mod trace;

// TODO: can we inline the vtable and get rid of the indirection? benchmark that though, might be a regression
pub type ObjectId = gc2::AllocId<&'static ObjectVTable>;

#[derive(Debug)]
pub struct Gc {
    /// The very first node of this [`Gc`]
    head: Option<NonNull<GcNode<()>>>,
    /// The last (-inserted) node of this [`Gc`]
    tail: Option<NonNull<GcNode<()>>>,
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

    #[cfg_attr(feature = "stress_gc", track_caller)]
    unsafe fn add(&mut self, ptr: NonNull<GcNode<()>>) -> Handle {
        debug!(?ptr, "alloc");
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

    #[cfg_attr(feature = "stress_gc", track_caller)]
    pub fn register<O: Object + 'static>(&mut self, value: O) -> Handle {
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
            // NB: take care to not create a reference to &GcNode<()> as that is undefined behavior
            // only place project the fields directly
            let flags = &(*ptr.as_ptr()).flags;
            let refcount = &(*ptr.as_ptr()).refcount;
            let next = (*ptr.as_ptr()).next;

            cur = next;

            // TODO: this refcount check in the if is probably not even necessary anymore;
            // we already trace existing `Persistent<T>`s, which marks them as visited.
            if !flags.is_marked() && refcount.get() == 0 {
                // Reference did not get marked during mark phase
                // Deallocate and unlink!

                // If this node is the head (i.e. oldest/first node) or there is no head,
                // set it to the next node.
                if self.head.map_or(true, |p| p == ptr) {
                    self.head = next;
                }

                // If this node is the tail (i.e. newest/most recently added node) or there is no tail,
                // set it to the last valid node.
                if self.tail.map_or(true, |p| p == ptr) {
                    self.tail = previous;
                }

                // Update last valid pointer to the next pointer
                if let Some(mut previous) = previous {
                    unsafe { previous.as_mut().next = next };
                }

                // Deallocate node.
                debug!(?ptr, "dealloc");

                unsafe {
                    drop_erased_gc_node(ptr.as_ptr());
                }

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
            let next = unsafe { (*node.as_ptr()).next };
            curr = next;

            unsafe {
                drop_erased_gc_node(node.as_ptr());
            }
        }
    }
}

/// # Safety
/// - The pointer must have been created using [`Box::into_raw`].
unsafe fn drop_erased_gc_node(s: *mut GcNode<()>) {
    ((*s).vtable.drop_boxed_gcnode)(s);
}

#[macro_export]
macro_rules! object_vtable_for_ty {
    ($ty:ty) => {
        const {
            use $crate::value::object::Object;

            &$crate::gc::handle::ObjectVTable {
                drop_boxed_gcnode: |ptr| unsafe {
                    drop(Box::from_raw(ptr.cast::<$crate::gc::handle::GcNode<$ty>>()));
                },
                trace: |ptr, ctxt| unsafe { <$ty as $crate::gc::trace::Trace>::trace(&*(ptr.cast::<$ty>()), ctxt) },
                debug_fmt: |ptr, f| unsafe { <$ty as std::fmt::Debug>::fmt(&*(ptr.cast::<$ty>()), f) },
                js_get_own_property: |ptr, scope, this, key| unsafe {
                    <$ty as Object>::get_own_property(&*(ptr.cast::<$ty>()), scope, this, key)
                },
                js_get_own_property_descriptor: |ptr, scope, key| unsafe {
                    <$ty as Object>::get_own_property_descriptor(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_get_property: |ptr, scope, this, key| unsafe {
                    <$ty as Object>::get_property(&*(ptr.cast::<$ty>()), scope, this, key)
                },
                js_get_property_descriptor: |ptr, scope, key| unsafe {
                    <$ty as Object>::get_property_descriptor(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_set_property: |ptr, scope, key, value| unsafe {
                    <$ty as Object>::set_property(&*(ptr.cast::<$ty>()), scope, key, value)
                },
                js_delete_property: |ptr, scope, key| unsafe {
                    <$ty as Object>::delete_property(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_set_prototype: |ptr, scope, proto| unsafe {
                    <$ty as Object>::set_prototype(&*(ptr.cast::<$ty>()), scope, proto)
                },
                js_get_prototype: |ptr, scope| unsafe { <$ty as Object>::get_prototype(&*(ptr.cast::<$ty>()), scope) },
                js_apply: |ptr, scope, callee, this, args| unsafe {
                    <$ty as Object>::apply(&*(ptr.cast::<$ty>()), scope, callee, this, args)
                },
                js_construct: |ptr, scope, callee, this, args| unsafe {
                    <$ty as Object>::construct(&*(ptr.cast::<$ty>()), scope, callee, this, args)
                },
                js_as_any: |ptr, vm| unsafe { <$ty as Object>::as_any(&*(ptr.cast::<$ty>()), vm) },
                js_internal_slots: |ptr, vm| unsafe {
                    <$ty as Object>::internal_slots(&*(ptr.cast::<$ty>()), vm)
                        .map(|v| v as *const dyn $crate::value::primitive::InternalSlots)
                },
                js_own_keys: |ptr, scope| unsafe { <$ty as Object>::own_keys(&*(ptr.cast::<$ty>()), scope) },
                js_type_of: |ptr, vm| unsafe { <$ty as Object>::type_of(&*(ptr.cast::<$ty>()), vm) },
            }
        }
    };
}

macro_rules! register_gc {
    ($ty:ty, $gc:expr, $val:expr) => {{
        assert!(
            std::mem::align_of::<$ty>() <= 8,
            "cannot register object of type {} because its alignment is {}",
            std::any::type_name::<$ty>(),
            std::mem::align_of::<$ty>()
        );

        let value: $ty = $val;
        #[allow(unused_unsafe)]
        let node = GcNode {
            vtable: &$crate::gc::handle::ObjectVTable {
                drop_boxed_gcnode: |ptr| unsafe {
                    drop(Box::from_raw(ptr.cast::<GcNode<$ty>>()));
                },
                trace: |ptr, ctxt| unsafe { <$ty as crate::gc::trace::Trace>::trace(&*(ptr.cast::<$ty>()), ctxt) },
                debug_fmt: |ptr, f| unsafe { <$ty as Debug>::fmt(&*(ptr.cast::<$ty>()), f) },
                js_get_own_property: |ptr, scope, this, key| unsafe {
                    <$ty as Object>::get_own_property(&*(ptr.cast::<$ty>()), scope, this, key)
                },
                js_get_own_property_descriptor: |ptr, scope, key| unsafe {
                    <$ty as Object>::get_own_property_descriptor(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_get_property: |ptr, scope, this, key| unsafe {
                    <$ty as Object>::get_property(&*(ptr.cast::<$ty>()), scope, this, key)
                },
                js_get_property_descriptor: |ptr, scope, key| unsafe {
                    <$ty as Object>::get_property_descriptor(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_set_property: |ptr, scope, key, value| unsafe {
                    <$ty as Object>::set_property(&*(ptr.cast::<$ty>()), scope, key, value)
                },
                js_delete_property: |ptr, scope, key| unsafe {
                    <$ty as Object>::delete_property(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_set_prototype: |ptr, scope, proto| unsafe {
                    <$ty as Object>::set_prototype(&*(ptr.cast::<$ty>()), scope, proto)
                },
                js_get_prototype: |ptr, scope| unsafe { <$ty as Object>::get_prototype(&*(ptr.cast::<$ty>()), scope) },
                js_apply: |ptr, scope, callee, this, args| unsafe {
                    <$ty as Object>::apply(&*(ptr.cast::<$ty>()), scope, callee, this, args)
                },
                js_construct: |ptr, scope, callee, this, args| unsafe {
                    <$ty as Object>::construct(&*(ptr.cast::<$ty>()), scope, callee, this, args)
                },
                js_as_any: |ptr, vm| unsafe { <$ty as Object>::as_any(&*(ptr.cast::<$ty>()), vm) },
                js_internal_slots: |ptr, vm| unsafe {
                    <$ty as Object>::internal_slots(&*(ptr.cast::<$ty>()), vm)
                        .map(|v| v as *const dyn crate::value::primitive::InternalSlots)
                },
                js_own_keys: |ptr, scope| unsafe { <$ty as Object>::own_keys(&*(ptr.cast::<$ty>()), scope) },
                js_type_of: |ptr, vm| unsafe { <$ty as Object>::type_of(&*(ptr.cast::<$ty>()), vm) },
            },
            flags: Default::default(),
            refcount: Default::default(),
            next: None,
            value: $crate::gc::handle::Align8(value),
        };
        let ptr: *mut GcNode<()> = Box::into_raw(Box::new(node)).cast();

        #[allow(unused_unsafe)]
        unsafe {
            $gc.add(NonNull::new(ptr).unwrap())
        }
    }};
}

/// # Safety
/// Implementors must provide a "correct" into_handle method
/// by returning a valid [`Handle`] living in the given linked list.
pub unsafe trait IntoHandle {
    #[cfg_attr(feature = "stress_gc", track_caller)]
    fn into_handle(self, gc: &mut Gc) -> Handle;
}

unsafe impl<T: Object + 'static> IntoHandle for T {
    #[cfg_attr(feature = "stress_gc", track_caller)]
    fn into_handle(self, gc: &mut Gc) -> Handle {
        register_gc!(Self, gc, self)
    }
}

#[cfg(test)]
mod tests {
    use crate::gc::handle::HandleFlagsInner;
    use crate::value::primitive::Number;
    use crate::value::{ExternalValue, Value};

    use super::*;

    #[test]
    fn simple() {
        unsafe {
            let mut gc = Gc::new();
            let _ = register_gc!(f64, gc, 123.4);
            let _ = register_gc!(bool, gc, true);
            gc.sweep();
            gc.sweep();
        }
    }

    #[test]
    fn gc_works() {
        unsafe {
            let mut gc = Gc::new();

            assert!(gc.node_count == 0);
            assert!(gc.head.is_none());
            assert!(gc.tail.is_none());

            let h1 = register_gc!(f64, gc, 123.0);

            assert!(gc.head == NonNull::new(h1.as_ptr()));
            assert!(gc.tail == NonNull::new(h1.as_ptr()));
            assert!(h1.next().is_none());
            assert!(!h1.flags().contains(HandleFlagsInner::MARKED_VISITED));
            assert!(gc.node_count == 1);

            let h2 = register_gc!(f64, gc, 123.4);

            assert!(gc.head == NonNull::new(h1.as_ptr()));
            assert!(gc.tail == NonNull::new(h2.as_ptr()));
            assert!(h1.next() == NonNull::new(h2.as_ptr()));
            assert!(!h2.flags().contains(HandleFlagsInner::MARKED_VISITED));
            assert!(gc.node_count == 2);

            (*h1.as_erased_ptr()).flags.mark();
            (*h2.as_erased_ptr()).flags.mark();

            assert!((*h1.as_erased_ptr()).flags.is_marked());
            assert!((*h2.as_erased_ptr()).flags.is_marked());

            gc.sweep();

            // nothing should have changed after GC sweep since all nodes were marked
            // they should be unmarked now though
            assert!(gc.head == NonNull::new(h1.as_ptr()));
            assert!(gc.tail == NonNull::new(h2.as_ptr()));
            assert!((*h1.as_erased_ptr()).next == NonNull::new(h2.as_ptr()));
            assert!(!(*h1.as_erased_ptr()).flags.is_marked());
            assert!(!(*h2.as_erased_ptr()).flags.is_marked());
            assert!(gc.node_count == 2);

            // add a third node now
            let h3 = register_gc!(bool, gc, true);

            assert!(gc.head == NonNull::new(h1.as_ptr()));
            assert!(gc.tail == NonNull::new(h3.as_ptr()));
            assert!((*h1.as_erased_ptr()).next == NonNull::new(h2.as_ptr()));
            assert!((*h2.as_erased_ptr()).next == NonNull::new(h3.as_ptr()));
            assert!(!(*h3.as_erased_ptr()).flags.is_marked());
            assert!(gc.node_count == 3);

            // ---

            // only mark second node
            (*h2.as_erased_ptr()).flags.mark();

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

            // test that ExternalValue::replace works
            {
                todo!();
                // let h4i: Handle = register_gc!(Value, gc, Value::Number(Number(123.4)));
                // let ext = ExternalValue::new(h4i);
                // assert_eq!(ext.inner(), &Value::Number(Number(123.4)));
                // ExternalValue::replace(&ext, Value::Boolean(true));
                // assert_eq!(ext.inner(), &Value::Boolean(true));
            }

            // lastly, test if Gc::drop works correctly. run under miri to see possible leaks
            register_gc!(bool, gc, false);
        }
    }
}
