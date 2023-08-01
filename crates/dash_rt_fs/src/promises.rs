use dash_rt::wrap_async;
use dash_vm::localscope::LocalScope;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyKey;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::value::Value;
use dash_vm::value::ValueContext;

pub fn init_module(sc: &mut LocalScope) -> Result<Value, Value> {
    let read_file_value = Function::new(sc, Some("readFile".into()), FunctionKind::Native(read_file));
    let read_file_value = sc.register(read_file_value);

    let module = NamedObject::new(sc);
    module.set_property(
        sc,
        PropertyKey::String("readFile".into()),
        PropertyValue::static_default(Value::Object(read_file_value)),
    )?;

    Ok(Value::Object(sc.register(module)))
}

fn read_file(cx: CallContext) -> Result<Value, Value> {
    let path = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    let path = ToString::to_string(&path);

    wrap_async(cx, tokio::fs::read_to_string(path), |sc, res| match res {
        Ok(s) => Ok(Value::String(s.into())),
        Err(e) => {
            let err = Error::new(sc, e.to_string());
            Err(Value::Object(sc.register(err)))
        }
    })
}
