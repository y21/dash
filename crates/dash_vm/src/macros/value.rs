#[macro_export]
macro_rules! throw {
    ($vm:expr) => {
        return Err({
            let mut vm = $vm;
            let err = $crate::value::error::Error::new(&mut vm, "Unnamed error");
            vm.gc_mut().register(err).into()
        })
    };
    ($vm:expr, $msg:expr) => {
        return Err({
            let mut vm = $vm;
            let err = $crate::value::error::Error::new(&mut vm, $msg);
            vm.gc_mut().register(err).into()
        })
    };
    ($vm:expr, $msg:expr, $($arg:expr),*) => {
        return Err({
            let mut vm = $vm;
            let err = $crate::value::error::Error::new(&mut vm, format!($msg, $($arg),*));
            vm.gc_mut().register(err).into()
        })
    };
}
