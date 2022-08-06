pub unsafe trait Trace {
    fn trace(&self);
}

unsafe impl<T: Trace> Trace for [T] {
    fn trace(&self) {
        for item in self {
            item.trace();
        }
    }
}

unsafe impl<T: Trace> Trace for Option<T> {
    fn trace(&self) {
        if let Some(t) = self {
            t.trace();
        }
    }
}

unsafe impl<T: Trace> Trace for Vec<T> {
    fn trace(&self) {
        self.as_slice().trace();
    }
}
