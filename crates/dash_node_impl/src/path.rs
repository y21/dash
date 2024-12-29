use std::path::{self, Path, PathBuf};

use dash_middle::interner::sym;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::{ExceptionContext, Unpack, Value, ValueKind};

use crate::state::state_mut;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let exports = NamedObject::new(sc);
    let parse_sym = state_mut(sc).sym.parse;
    let parse_path = register_native_fn(sc, parse_sym, parse_path);
    let join_path = register_native_fn(sc, sym::join, join_path);
    exports.set_property(parse_sym.into(), PropertyValue::static_default(parse_path.into()), sc)?;
    exports.set_property(sym::join.into(), PropertyValue::static_default(join_path.into()), sc)?;

    Ok(sc.register(exports).into())
}

fn parse_path(cx: CallContext) -> Result<Value, Value> {
    let path = cx.args.first().or_type_err(cx.scope, "Missing path to path")?;
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
        dir_sym.into(),
        PropertyValue::static_default(Value::string(dir.into())),
        cx.scope,
    )?;
    Ok(cx.scope.register(object).into())
}

fn join_path(cx: CallContext) -> Result<Value, Value> {
    let mut path = PathBuf::new();

    for arg in &cx.args {
        let value = match arg.unpack() {
            ValueKind::String(s) => s.res(cx.scope),
            other => throw!(
                cx.scope,
                TypeError,
                "expected string argument to path.join, got {:?}",
                other
            ),
        };

        for segment in value.split(path::MAIN_SEPARATOR) {
            match segment {
                ".." => drop(path.pop()),
                "." => {}
                _ => path.push(segment),
            }
        }
    }

    Ok(Value::string(
        cx.scope.intern(path.display().to_string().as_str()).into(),
    ))
}
