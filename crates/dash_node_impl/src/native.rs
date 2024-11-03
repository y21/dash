use dash_vm::localscope::LocalScope;
use dash_vm::value::object::NamedObject;
use dash_vm::value::string::JsString;
use dash_vm::value::Value;

use crate::state::state_mut;

macro_rules! check_module {
    (
        $arg:expr;
        $sc:expr;
        $(
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
        state.sym.fs => (state_mut(sc).fs_cache, dash_rt_fs::sync::init_module),
        state.sym.fetch => (state_mut(sc).fetch_cache, dash_rt_fetch::init_module),
        state.sym.path => (state_mut(sc).path_cache, crate::path::init_module),
        state.sym.events => (state_mut(sc).events_cache, crate::events::init_module),
        state.sym.stream => (state_mut(sc).stream_cache, crate::stream::init_module),
        state.sym.http => (state_mut(sc).http_cache, init_dummy_empty_module),
        state.sym.https => (state_mut(sc).https_cache, init_dummy_empty_module),
        state.sym.url => (state_mut(sc).url_cache, init_dummy_empty_module),
        state.sym.zlib => (state_mut(sc).zlib_cache, init_dummy_empty_module),
        state.sym.punycode => (state_mut(sc).punycode_cache, init_dummy_empty_module),
        state.sym.util => (state_mut(sc).util_cache, crate::util::init_module)
    }
}

fn init_dummy_empty_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let exports = NamedObject::new(sc);
    Ok(Value::object(sc.register(exports)))
}
