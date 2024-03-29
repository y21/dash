use dash_vm::localscope::LocalScope;
use dash_vm::value::string::JsString;
use dash_vm::value::Value;

use crate::state::state_mut;

macro_rules! check_module {
    (
        $arg:expr;
        $sc:expr;
        $(
            #[$($attr:meta)*]
            $sym:expr => ($cache:expr, $init:expr)
        ),*
    ) => {
        let arg = $arg;
        if false { loop {} }
        $(
            else if arg == $sym {
                if let Some(val) = $cache.get() {
                    Ok(Some(val.clone()))
                } else {
                    let val = $init($sc)?;
                    $cache.set(val.clone()).unwrap();
                    Ok(Some(val))
                }
            }
        )*
        else {
            Ok(None)
        }
    };
}

pub fn load_native_module(sc: &mut LocalScope<'_>, arg: JsString) -> Result<Option<Value>, Value> {
    let state = state_mut(sc);

    check_module! {
        arg.sym();
        sc;
        #[cfg(feature = "fs")]
        state.sym.fs => (state_mut(sc).fs_cache, dash_rt_fs::sync::init_module),
        #[cfg(feature = "fetch")]
        state.sym.fetch => (state_mut(sc).fetch_cache, dash_rt_fetch::init_module)
    }
}
