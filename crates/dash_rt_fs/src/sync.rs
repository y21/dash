use std::path::Path;
use std::slice;

use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{OrdObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::typedarray::TypedArray;
use dash_vm::value::{Value, ValueContext};

pub fn init_module(sc: &mut LocalScope) -> Result<Value, Value> {
    let read_file_sync_sym = sc.intern("readFileSync");
    let write_file_sync_sym = sc.intern("writeFileSync");
    let read_file_sync_value = register_native_fn(sc, read_file_sync_sym, read_file_sync);
    let write_file_sync_value = register_native_fn(sc, write_file_sync_sym, write_file_sync);

    let module = OrdObject::new(sc);
    module.set_property(
        read_file_sync_sym.to_key(sc),
        PropertyValue::static_default(Value::object(read_file_sync_value)),
        sc,
    )?;
    module.set_property(
        write_file_sync_sym.to_key(sc),
        PropertyValue::static_default(Value::object(write_file_sync_value)),
        sc,
    )?;

    Ok(Value::object(sc.register(module)))
}

fn read_file_sync(cx: CallContext) -> Result<Value, Value> {
    let path = cx
        .args
        .first()
        .unwrap_or_undefined()
        .to_js_string(cx.scope)?
        .res(cx.scope)
        .to_owned();

    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Value::string(cx.scope.intern(s.as_ref()).into())),
        Err(err) => {
            let err = Error::new(cx.scope, err.to_string());
            Err(Value::object(cx.scope.register(err)))
        }
    }
}

fn write_file_sync(cx: CallContext) -> Result<Value, Value> {
    let [path, buf] = *cx.args else {
        throw!(
            cx.scope,
            Error,
            "Invalid arguments passed to fs.writeFileSync(path, buf)"
        )
    };
    let path = path.to_js_string(cx.scope)?;
    if let Some(array) = buf.extract::<TypedArray>(cx.scope) {
        let path = Path::new(path.res(cx.scope));

        let storage = array.arraybuffer(cx.scope).storage();
        // SAFETY: Cell<u8> has the same layout as u8
        let view = unsafe { slice::from_raw_parts(storage.as_ptr().cast::<u8>(), storage.len()) };

        match std::fs::write(path, view) {
            Ok(()) => Ok(Value::undefined()),
            Err(err) => {
                let err = Error::new(cx.scope, err.to_string());
                Err(Value::object(cx.scope.register(err)))
            }
        }
    } else {
        throw!(cx.scope, TypeError, "Invalid source passed to fs.writeFileSync")
    }
}
