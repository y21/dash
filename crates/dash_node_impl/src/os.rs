use std::env;

use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{Object, OrdObject, PropertyValue};
use dash_vm::value::propertykey::ToPropertyKey;

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let exports = OrdObject::new(sc);
    let NodeSymbols { tmpdir: tmpdir_sym, .. } = state_mut(sc).sym;
    let tmpdir_fn = register_native_fn(sc, tmpdir_sym, tmpdir);
    exports.set_property(
        tmpdir_sym.to_key(sc),
        PropertyValue::static_default(tmpdir_fn.into()),
        sc,
    )?;
    Ok(sc.register(exports).into())
}

fn tmpdir(cx: CallContext) -> Result<Value, Value> {
    let path = env::temp_dir();
    Ok(Value::string(
        cx.scope.intern(path.display().to_string().as_str()).into(),
    ))
}
