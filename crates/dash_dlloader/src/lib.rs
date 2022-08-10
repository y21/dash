use std::mem::ManuallyDrop;

use dash_vm::local::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::value::Value;
use libloading::Library;

type InitFunction = unsafe extern "C" fn(*mut CallContext, *mut Result<Value, Value>);

#[macro_export]
macro_rules! dashdl {
    ($fun:path) => {
        #[no_mangle]
        pub unsafe extern "C" fn dashjs_init_module(
            cx: *mut ::dash_vm::value::function::native::CallContext,
            ret: *mut Result<::dash_vm::value::Value, ::dash_vm::value::Value>,
        ) {
            ret.write($fun(&mut *cx));
        }
    };
}

pub fn load_sync(mut cx: CallContext) -> Result<Value, Value> {
    let path = match cx.args.first() {
        Some(first) => first,
        None => throw!(cx.scope, "Missing path to dynamic library"),
    };

    let path = ValueConversion::to_string(path, cx.scope)?;

    unsafe {
        let lib = match Library::new(path.as_ref()) {
            // TODO: Currently we (intentionally) leak all dlopen'd handles, because we don't know exactly when we should close it
            Ok(lib) => ManuallyDrop::new(lib),
            Err(err) => throw!(cx.scope, "{}", err),
        };

        let init: libloading::Symbol<InitFunction> = match lib.get(b"dashjs_init_module\0") {
            Ok(sym) => sym,
            Err(err) => throw!(cx.scope, "{}", err),
        };

        let mut ret = Ok(Value::undefined());
        init(&mut cx, &mut ret);
        ret
    }
}

pub fn import_dl(scope: &mut LocalScope) -> Result<Value, Value> {
    let object = NamedObject::new(scope);
    let load_sync = Function::new(scope, Some("loadSync".into()), FunctionKind::Native(load_sync));
    let load_sync = scope.register(load_sync);
    object.set_property(
        scope,
        "loadSync".into(),
        PropertyValue::Static(Value::Object(load_sync)),
    )?;

    let object = scope.register(object);
    Ok(Value::Object(object))
}
