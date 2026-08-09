use std::path::{self, Path, PathBuf};

use dash_middle::interner::sym;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{Object, OrdObject, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::{ExceptionContext, Unpack, Value, ValueKind};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let exports = OrdObject::new(sc);
    let NodeSymbols {
        parse: parse_sym,
        isAbsolute: is_absolute_sym,
        dirname: dirname_sym,
        basename: basename_sym,
        ..
    } = state_mut(sc).sym;
    let parse_path = register_native_fn(sc, parse_sym, false, parse_path);
    let join_path = register_native_fn(sc, sym::join, false, join_path);
    let resolve_path = register_native_fn(sc, sym::resolve, false, resolve_path);
    let is_absolute_path = register_native_fn(sc, is_absolute_sym, false, is_absolute_path);
    let dirname = register_native_fn(sc, dirname_sym, false, dirname);
    let basename = register_native_fn(sc, basename_sym, false, basename);

    exports.set_property(
        parse_sym.to_key(sc),
        PropertyValue::static_default(parse_path.into()),
        sc,
    )?;
    exports.set_property(
        sym::join.to_key(sc),
        PropertyValue::static_default(join_path.into()),
        sc,
    )?;
    exports.set_property(
        sym::resolve.to_key(sc),
        PropertyValue::static_default(resolve_path.into()),
        sc,
    )?;
    exports.set_property(
        is_absolute_sym.to_key(sc),
        PropertyValue::static_default(is_absolute_path.into()),
        sc,
    )?;
    exports.set_property(
        dirname_sym.to_key(sc),
        PropertyValue::static_default(dirname.into()),
        sc,
    )?;
    exports.set_property(
        basename_sym.to_key(sc),
        PropertyValue::static_default(basename.into()),
        sc,
    )?;

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
    let object = OrdObject::new(cx.scope);
    let object = cx.scope.register(object);
    let dir_sym = state_mut(cx.scope).sym.dir;
    object.set_property(
        dir_sym.to_key(cx.scope),
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

        for component in Path::new(value).components() {
            match component {
                path::Component::CurDir => {}
                path::Component::ParentDir => {
                    path.pop();
                }
                path::Component::Prefix(_) | path::Component::RootDir | path::Component::Normal(_) => {
                    path.push(component)
                }
            }
        }
    }

    Ok(Value::string(
        cx.scope.intern(path.display().to_string().as_str()).into(),
    ))
}

fn resolve_path(cx: CallContext) -> Result<Value, Value> {
    let mut path = PathBuf::new();

    for arg in cx.args.iter().rev() {
        let value = arg.to_js_string(cx.scope)?.res(cx.scope);
        if path.is_empty() {
            path = Path::new(value).to_path_buf();
        } else {
            path = Path::new(value).join(path);
        }
    }

    match path.canonicalize() {
        Ok(path) => Ok(Value::string(
            cx.scope.intern(path.display().to_string().as_str()).into(),
        )),
        Err(err) => throw!(cx.scope, Error, "failed to canonicalize path: {}", err),
    }
}

fn is_absolute_path(cx: CallContext) -> Result<Value, Value> {
    let path = cx
        .args
        .first()
        .or_type_err(cx.scope, "Missing path to path.isAbsolute")?
        .to_js_string(cx.scope)?;
    let path = Path::new(path.res(cx.scope));
    Ok(Value::boolean(path.is_absolute()))
}

fn dirname(cx: CallContext) -> Result<Value, Value> {
    let path = cx
        .args
        .first()
        .or_type_err(cx.scope, "Missing path to path.dirname")?
        .to_js_string(cx.scope)?;
    let path = Path::new(path.res(cx.scope));
    let dir = match path.parent() {
        Some(parent) => match parent.to_str() {
            Some(s) => s.to_owned(),
            None => throw!(cx.scope, Error, "invalid utf-8 in path"),
        },
        None => "/".to_owned(),
    };
    Ok(Value::string(cx.scope.intern(dir).into()))
}

fn basename(cx: CallContext) -> Result<Value, Value> {
    let path = cx
        .args
        .first()
        .or_type_err(cx.scope, "Missing path to path.basename")?
        .to_js_string(cx.scope)?;

    let suffix = cx.args.get(1);

    let path = Path::new(path.res(cx.scope));
    let base = match path.file_name() {
        Some(name) => match name.to_str() {
            Some(s) => s.to_owned(),
            None => throw!(cx.scope, Error, "invalid utf-8 in path"),
        },
        None => throw!(cx.scope, Error, "path has no basename"),
    };

    let base = if let Some(suffix) = suffix {
        let suffix = suffix.to_js_string(cx.scope)?.res(cx.scope);
        base.strip_suffix(suffix).unwrap_or(&base)
    } else {
        &base
    };

    Ok(Value::string(cx.scope.intern(base).into()))
}
