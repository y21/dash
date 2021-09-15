/// A heap allocated object that can be transferred through WebAssembly boundaries
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Handle<T: ?Sized>(*mut T);

impl<T> Handle<T> {
    pub fn new(data: T) -> Self {
        Self(Box::leak(Box::new(data)))
    }
}

impl<T: ?Sized> Handle<T> {
    pub unsafe fn as_ref(&self) -> &T {
        &*self.0
    }

    pub unsafe fn drop(self) {
        drop(Box::from_raw(self.0));
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0
    }

    pub unsafe fn into_box(self) -> Box<T> {
        Box::from_raw(self.0)
    }
}

/// A pointer to a heap allocated object that can be transferred through WebAssembly boundaries
#[repr(C)]
pub struct HandleRef<T: ?Sized>(*mut T);
impl<T: ?Sized> HandleRef<T> {
    pub fn from_ptr(ptr: *mut T) -> Self {
        Self(ptr)
    }

    pub unsafe fn as_ref(&self) -> &T {
        &*self.0
    }

    pub unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.0
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0
    }
}

impl<T> From<*mut T> for HandleRef<T> {
    fn from(ptr: *mut T) -> Self {
        Self::from_ptr(ptr)
    }
}
