#[macro_export]
macro_rules! throw {
    ($vm:expr, $err:ident, $msg:expr) => {
        return Err({
            // for some reason it warns about unused mut when it really is required, remove when fixed. (rust#105149)
            #[allow(unused_mut)]
            let mut vm = $vm;
            let err = $crate::value::error::$err::new(&mut vm as &mut $crate::localscope::LocalScope<'_>, $msg);
            Value::Object(vm.gc_mut().register(err)).into()
        })
    };
    ($vm:expr, $err:ident, $msg:expr, $($arg:expr),*) => {
        return Err({
            // for some reason it warns about unused mut when it really is required, remove when fixed. (rust#105149)
            #[allow(unused_mut)]
            let mut vm: &mut $crate::localscope::LocalScope<'_> = $vm;
            let err = $crate::value::error::$err::new(vm, format!($msg, $($arg),*));
            Value::Object(vm.gc_mut().register(err)).into()
        })
    };
}
