use dash_core::vm::local::LocalScope;
use dash_core::vm::value::error::Error;
use dash_core::vm::value::object::NamedObject;
use dash_core::vm::value::Value as DashValue;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue as WasmValue;

use crate::externalfunction::ExternalFunction;
use crate::jsvalue::JsValue;

pub fn wasm_value_from_dash_value(_scope: &mut LocalScope, value: DashValue) -> Result<wasm_bindgen::JsValue, String> {
    match value {
        DashValue::Undefined(_) => Ok(WasmValue::UNDEFINED),
        DashValue::Null(_) => Ok(WasmValue::NULL),
        DashValue::Boolean(b) => Ok(WasmValue::from_bool(b)),
        DashValue::Number(n) => Ok(WasmValue::from_f64(n)),
        DashValue::String(s) => Ok(WasmValue::from_str(&s)),
        DashValue::Object(o) => Ok(WasmValue::from(JsValue::from(DashValue::Object(o)))),
        DashValue::Symbol(_) => Err("Unhandled symbol".into()),
        DashValue::External(_) => Err("Unhandled external".into()),
    }
}

pub fn dash_value_from_wasm_value(scope: &mut LocalScope, value: WasmValue) -> Result<DashValue, String> {
    if let Some(value) = value.as_f64() {
        Ok(DashValue::Number(value))
    } else if let Some(value) = value.as_bool() {
        Ok(DashValue::Boolean(value))
    } else if let Some(value) = value.as_string() {
        Ok(DashValue::String(value.into()))
    } else if value.is_undefined() {
        Ok(DashValue::undefined())
    } else if value.is_object() {
        // special case error objects
        if value.is_instance_of::<js_sys::Error>() {
            let key = wasm_bindgen::JsValue::from_str("message");
            let message = match js_sys::Reflect::get(&value, &key).map(|v| v.as_string()) {
                Ok(Some(message)) => message,
                _ => return Err("Failed to read message property".into()),
            };
            let error = Error::new(scope, message);
            return Ok(DashValue::Object(scope.register(error)));
        }

        let source = js_sys::Object::from(value);
        let dest = {
            let obj = NamedObject::new(scope);
            scope.register(obj)
        };

        let entries = js_sys::Object::entries(&source);
        for entry in entries.iter() {
            let entry = js_sys::Array::from(&entry);

            let key = match entry.at(0).as_string() {
                Some(key) => key,
                None => return Err("Non-string object keys not supported".into()),
            };
            let value = entry.at(1);

            let value = dash_value_from_wasm_value(scope, value)?;

            dest.set_property(scope, key.into(), value)
                .map_err(|_| "Failed to set property")?;
        }

        Ok(DashValue::Object(dest))
    } else if value.is_null() {
        Ok(DashValue::null())
    } else if value.is_function() {
        let fun = ExternalFunction::new(js_sys::Function::from(value), NamedObject::new(scope));
        Ok(DashValue::Object(scope.register(fun)))
    } else {
        Err("Invalid value".into())
    }
}
