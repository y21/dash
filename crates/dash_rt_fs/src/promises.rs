use dash_rt::wrap_async;
use dash_vm::localscope::LocalScope;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::{Value, ValueContext};

pub fn init_module(sc: &mut LocalScope) -> Result<Value, Value> {
    let name = sc.intern("readFile");
    let read_file_value = Function::new(sc, Some(name.into()), FunctionKind::Native(read_file));
    let read_file_value = sc.register(read_file_value);

    let module = NamedObject::new(sc);
    module.set_property(
        name.into(),
        PropertyValue::static_default(Value::object(read_file_value)),
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
