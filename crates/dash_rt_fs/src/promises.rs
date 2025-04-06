use std::cell::Cell;
use std::path::PathBuf;

use dash_rt::wrap_async;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::typedarray::TypedArray;
use dash_vm::value::{Value, ValueContext};

pub fn init_module(sc: &mut LocalScope) -> Result<Value, Value> {
    let read_file_k = sc.intern("readFile");
    let write_file_k = sc.intern("writeFile");
    let read_file_value = register_native_fn(sc, read_file_k, read_file);
    let write_file_value = register_native_fn(sc, write_file_k, write_file);

    let module = NamedObject::new(sc);
    module.set_property(
        read_file_k.to_key(sc),
        PropertyValue::static_default(read_file_value.into()),
        sc,
    )?;
    module.set_property(
        write_file_k.to_key(sc),
        PropertyValue::static_default(write_file_value.into()),
        sc,
    )?;

    Ok(Value::object(sc.register(module)))
}

fn read_file(cx: CallContext) -> Result<Value, Value> {
    let path = cx
        .args
        .first()
        .unwrap_or_undefined()
        .to_js_string(cx.scope)?
        .res(cx.scope)
        .to_owned();

    wrap_async(cx, tokio::fs::read_to_string(path), |sc, res| match res {
        Ok(s) => Ok(Value::string(sc.intern(s.as_ref()).into())),
        Err(e) => {
            let err = Error::new(sc, e.to_string());
            Err(Value::object(sc.register(err)))
        }
    })
}

fn write_file(cx: CallContext) -> Result<Value, Value> {
    let [path, buf] = *cx.args else {
        throw!(
            cx.scope,
            Error,
            "Invalid arguments passed to fs.writeFileSync(path, buf)"
        )
    };
    let path = path.to_js_string(cx.scope)?;
    if let Some(array) = buf.extract::<TypedArray>(cx.scope) {
        let path = PathBuf::from(path.res(cx.scope));

        let storage = array
            .arraybuffer(cx.scope)
            .storage()
            .iter()
            .map(Cell::get)
            .collect::<Vec<_>>();

        wrap_async(cx, tokio::fs::write(path, storage), |sc, res| match res {
            Ok(()) => Ok(Value::undefined()),
            Err(err) => {
                let err = Error::new(sc, err.to_string());
                Err(Value::object(sc.register(err)))
            }
        })
    } else {
        throw!(cx.scope, TypeError, "Invalid source passed to fs.writeFileSync")
    }
}
