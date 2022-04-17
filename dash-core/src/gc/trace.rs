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
