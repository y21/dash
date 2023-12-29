use dash_vm::gc::handle::Handle;
use dash_vm::gc::trace::{Trace, TraceCtxt};
use dash_vm::localscope::LocalScope;
use dash_vm::value::object::{NamedObject, Object, PropertyKey, PropertyValue};
use dash_vm::value::{Typeof, Unrooted, Value};

use crate::util::{dash_value_from_wasm_value, wasm_value_from_dash_value};

#[derive(Debug)]
pub struct ExternalFunction(js_sys::Function, NamedObject);

impl ExternalFunction {
    pub fn new(fun: js_sys::Function, obj: NamedObject) -> Self {
        Self(fun, obj)
    }
}

unsafe impl Trace for ExternalFunction {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        self.1.trace(cx);
    }
}

impl Object for ExternalFunction {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        self.1.get_own_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        self.1.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        self.1.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.1.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.1.get_prototype(sc)
    }

    fn apply(&self, scope: &mut LocalScope, _callee: Handle, _this: Value, args: Vec<Value>) -> Result<Value, Value> {
        let this = wasm_bindgen::JsValue::UNDEFINED;

        let args = args
            .into_iter()
            .map(|v| wasm_value_from_dash_value(scope, v))
            .collect::<Result<js_sys::Array, _>>()
            .map_err(|e| Value::String(e.into()))?;

        match self.0.apply(&this, &args) {
            Ok(v) => dash_value_from_wasm_value(scope, v).map_err(|e| Value::String(e.into())),
            Err(v) => Err(dash_value_from_wasm_value(scope, v).map_err(|e| Value::String(e.into()))?),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.1.own_keys()
    }

    fn type_of(&self) -> Typeof {
        Typeof::Function
    }
}
