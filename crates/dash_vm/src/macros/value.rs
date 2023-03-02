#[macro_export]
macro_rules! throw {
    ($vm:expr, $err:ident, $msg:expr) => {
        return Err({
            let mut vm = $vm;
            let err = $crate::value::error::$err::new(&mut vm, $msg);
            vm.gc_mut().register(err).into()
        })
    };
    ($vm:expr, $err:ident, $msg:expr, $($arg:expr),*) => {
        return Err({
            let mut vm = $vm;
            let err = $crate::value::error::$err::new(&mut vm, format!($msg, $($arg),*));
            vm.gc_mut().register(err).into()
        })
    };
}
