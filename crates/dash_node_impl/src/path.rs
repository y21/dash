use std::path::Path;

use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::Value;

use crate::state::state_mut;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let exports = NamedObject::new(sc);
    let parse_sym = state_mut(sc).sym.parse;
    let parse_path = Function::new(sc, Some(parse_sym.into()), FunctionKind::Native(parse_path));
    let parse_path = sc.register(parse_path);
    exports.set_property(sc, parse_sym.into(), PropertyValue::static_default(parse_path.into()))?;

    Ok(sc.register(exports).into())
}

fn parse_path(cx: CallContext) -> Result<Value, Value> {
    let Some(path) = cx.args.first() else {
        throw!(cx.scope, Error, "missing path to parse");
    };
    let path = path.to_js_string(cx.scope)?;
    let path = Path::new(path.res(cx.scope));
    let dir = if path.is_dir() {
        path.to_str()
    } else {
        path.parent().and_then(Path::to_str)
    };
    let dir = match dir {
        Some(path) => cx.scope.intern(path.to_owned()),
        None => throw!(cx.scope, Error, "malformed path"),
    };
    let object = NamedObject::new(cx.scope);
    let object = cx.scope.register(object);
    let dir_sym = state_mut(cx.scope).sym.dir;
    object.set_property(
        cx.scope,
        dir_sym.into(),
        PropertyValue::static_default(Value::string(dir.into())),
    )?;
    Ok(cx.scope.register(object).into())
}
