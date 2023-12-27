use dash_vm::localscope::LocalScope;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyKey, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::{Value, ValueContext};

pub fn init_module(sc: &mut LocalScope) -> Result<Value, Value> {
    let name = sc.intern("readFile");
    let read_file_value = Function::new(sc, Some(name.into()), FunctionKind::Native(read_file));
    let read_file_value = sc.register(read_file_value);

    let module = NamedObject::new(sc);
    module.set_property(
        sc,
        name.into(),
        PropertyValue::static_default(Value::Object(read_file_value)),
    )?;

    Ok(Value::Object(sc.register(module)))
}

fn read_file(cx: CallContext) -> Result<Value, Value> {
    let path = cx
        .args
        .first()
        .unwrap_or_undefined()
        .to_js_string(cx.scope)?
        .res(cx.scope)
        .to_owned();

    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Value::String(cx.scope.intern(s.as_ref()).into())),
        Err(err) => {
            let err = Error::new(cx.scope, err.to_string());
            Err(Value::Object(cx.scope.register(err)))
        }
    }
}
