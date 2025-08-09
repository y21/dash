use std::cell::Cell;
use std::path::PathBuf;

use dash_rt::wrap_async;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{Object, OrdObject, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::typedarray::TypedArray;
use dash_vm::value::{Value, ValueContext};

pub fn init_module(sc: &mut LocalScope) -> Result<Value, Value> {
    let read_file_k = sc.intern("readFile");
    let write_file_k = sc.intern("writeFile");
    let read_file_value = register_native_fn(sc, read_file_k, read_file);
    let write_file_value = register_native_fn(sc, write_file_k, write_file);

    let module = OrdObject::new(sc);
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

fn read_file(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let path = cx
        .args
        .first()
        .unwrap_or_undefined()
        .to_js_string(scope)?
        .res(scope)
        .to_owned();

    wrap_async(scope, tokio::fs::read_to_string(path), |sc, res| match res {
        Ok(s) => Ok(Value::string(sc.intern(s.as_ref()).into())),
        Err(e) => {
            let err = Error::new(sc, e.to_string());
            Err(Value::object(sc.register(err)))
        }
    })
}

fn write_file(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let [path, buf] = *cx.args else {
        throw!(scope, Error, "Invalid arguments passed to fs.writeFileSync(path, buf)")
    };
    let path = path.to_js_string(scope)?;
    if let Some(array) = buf.extract::<TypedArray>(scope) {
        let path = PathBuf::from(path.res(scope));

        let storage = array
            .arraybuffer(scope)
            .storage()
            .iter()
            .map(Cell::get)
            .collect::<Vec<_>>();

        wrap_async(scope, tokio::fs::write(path, storage), |sc, res| match res {
            Ok(()) => Ok(Value::undefined()),
            Err(err) => {
                let err = Error::new(sc, err.to_string());
                Err(Value::object(sc.register(err)))
            }
        })
    } else {
        throw!(scope, TypeError, "Invalid source passed to fs.writeFileSync")
    }
}
