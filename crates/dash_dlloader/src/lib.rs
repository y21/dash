#[macro_use]
extern crate dlopen_derive;

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
use dlopen::wrapper::Container;
use dlopen::wrapper::WrapperApi;

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

#[derive(WrapperApi)]
pub struct Library {
    dashjs_init_module: unsafe fn(cx: *mut CallContext, ret: *mut Result<Value, Value>),
}

pub fn load_sync(mut cx: CallContext) -> Result<Value, Value> {
    let path = match cx.args.first() {
        Some(first) => first,
        None => throw!(cx.scope, "Missing path to dynamic library"),
    };

    let path = ValueConversion::to_string(path, cx.scope)?;

    let lib = match unsafe { Container::<Library>::load(path.as_ref()) } {
        Ok(lib) => lib,
        Err(err) => {
            throw!(cx.scope, "{}", err)
        }
    };

    let mut ret = Ok(Value::undefined());
    unsafe { lib.dashjs_init_module(&mut cx, &mut ret) };
    ret
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
