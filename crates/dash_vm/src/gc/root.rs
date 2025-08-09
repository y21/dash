use std::cell::{RefCell, UnsafeCell};
use std::ops::Deref;

use crate::gc::ObjectId;
use crate::gc::trace::Trace;
use crate::value::Value;
use crate::value::function::native::CallContext;

#[derive(PartialEq, Eq)]
pub enum ExternalRoot {
    Value(Value),
    Trace(*const dyn Trace),
}

pub trait IntoExternalRoot {
    fn into_external_root(&self) -> ExternalRoot;
}

impl IntoExternalRoot for Value {
    fn into_external_root(&self) -> ExternalRoot {
        ExternalRoot::Value(*self)
    }
}

impl IntoExternalRoot for ObjectId {
    fn into_external_root(&self) -> ExternalRoot {
        ExternalRoot::Value(Value::object(*self))
    }
}

impl<T: Trace + 'static> IntoExternalRoot for Vec<T> {
    fn into_external_root(&self) -> ExternalRoot {
        ExternalRoot::Trace(self as *const dyn Trace)
    }
}
impl<T: Trace + 'static> IntoExternalRoot for RefCell<T> {
    fn into_external_root(&self) -> ExternalRoot {
        ExternalRoot::Trace(self as *const dyn Trace)
    }
}
impl IntoExternalRoot for CallContext {
    fn into_external_root(&self) -> ExternalRoot {
        ExternalRoot::Trace(self as *const dyn Trace)
    }
}

pub struct RootStack {
    roots: UnsafeCell<Vec<ExternalRoot>>,
}

impl RootStack {
    pub fn new() -> Self {
        Self {
            roots: UnsafeCell::new(Vec::new()),
        }
    }
    pub fn push(&self, root: ExternalRoot) {
        unsafe { (*self.roots.get()).push(root) };
    }
    pub fn pop(&self) -> Option<ExternalRoot> {
        unsafe { (*self.roots.get()).pop() }
    }
    pub fn reserve(&self, additional: usize) {
        unsafe { (*self.roots.get()).reserve(additional) };
    }
}

pub struct LocalRootGuard<T> {
    #[doc(hidden)]
    pub value: T,
    #[doc(hidden)]
    pub stack: *const RootStack,
}

impl<T> Deref for LocalRootGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Drop for LocalRootGuard<T> {
    fn drop(&mut self) {
        unsafe { (*self.stack).pop() };
    }
}

macro_rules! letroot {
    (@count let $_:ident = $__:expr; $(let $name:ident = $init:expr;)*) => {
        1 $(+ letroot!(@count let $name = $init;))*
    };
    (@count) => { 0 };
    ($vm:expr, $(let $name:ident = $init:expr;)*) => {
        $vm.rootstack.reserve(letroot!(@count $(let $name = $init;)*));
        $(
            let $name = $crate::gc::root::LocalRootGuard {
                value: $init,
                stack: &$vm.rootstack as *const _,
            };
            let $name = &$name;
            $vm.rootstack.push($crate::gc::root::IntoExternalRoot::into_external_root(&$name.value));
        )*
    };
}

pub unsafe trait Rooted {}
unsafe impl<T> Rooted for LocalRootGuard<T> {}
unsafe impl<T: Rooted> Rooted for &T {}

pub fn assert_rooted(_: impl Rooted) {}

pub trait RootedFn<A, R>: Rooted {
    fn call(&self, args: A) -> R;
}

macro_rules! rootclosure {
    (
        $(<$($lt:lifetime),*>)?
        $($capture_ident:ident: $capture_ty:ty,)*
        |$($ident:ident: $ty:ty),*| -> $ret:ty $body:block
    ) => {{
        struct AnonClosure$(<$($lt),*>)? {
            $(pub $capture_ident: $capture_ty,)*
        }

        if false {
            $(
                $crate::gc::root::assert_rooted($capture_ident);
            )*
        }

        // SAFETY: all fields are rooted, so the closure itself is too
        unsafe impl$(<$($lt).*>)? $crate::gc::root::Rooted for AnonClosure$(<$($lt),*>)? {}

        impl$(<$($lt),*>)? $crate::gc::root::RootedFn<($($ty,)*), $ret> for AnonClosure$(<$($lt),*>)? {
            fn call(&self, args: ($($ty,)*)) -> $ret {
                $(
                    let $capture_ident = self.$capture_ident;
                )*
                let ($($ident,)*) = args;
                $body
            }
        }

        AnonClosure {
            $($capture_ident),*
        }
    }};
}

pub(crate) use {letroot, rootclosure};
