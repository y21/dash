use dash_vm::value::object::PropertyValue;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::value::{Typeof, Value as DashValue};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::externalvm::ExternalVm;
use crate::util::dash_value_from_wasm_value;

#[wasm_bindgen]
pub enum ExternalTypeof {
    Undefined,
    Object,
    Boolean,
    Number,
    Bigint,
    String,
    Symbol,
    Function,
}

impl From<Typeof> for ExternalTypeof {
    fn from(t: Typeof) -> Self {
        match t {
            Typeof::Undefined => Self::Undefined,
            Typeof::Object => Self::Object,
            Typeof::Boolean => Self::Boolean,
            Typeof::Number => Self::Number,
            Typeof::Bigint => Self::Bigint,
            Typeof::String => Self::String,
            Typeof::Symbol => Self::Symbol,
            Typeof::Function => Self::Function,
        }
    }
}

#[wasm_bindgen]
pub struct JsValue(DashValue);

impl From<DashValue> for JsValue {
    fn from(value: DashValue) -> Self {
        JsValue(value)
    }
}

#[wasm_bindgen]
impl JsValue {
    #[wasm_bindgen(constructor)]
    pub fn new(vm: &mut ExternalVm, value: wasm_bindgen::JsValue) -> Result<JsValue, String> {
        vm.with_scope(|scope| {
            let value = dash_value_from_wasm_value(scope, value)?;
            Ok(JsValue(value))
        })
    }

    pub fn to_js_string(&self, vm: &mut ExternalVm) -> Result<String, JsValue> {
        vm.with_scope(|scope| self.0.to_string(scope).map_err(JsValue).map(|s| s.as_ref().into()))
    }

    pub fn set_property(&self, vm: &mut ExternalVm, key: String, value: JsValue) -> Result<(), JsValue> {
        vm.with_scope(|scope| {
            let value = value.0;
            self.0
                .set_property(scope, key.into(), PropertyValue::static_default(value))
                .map_err(JsValue)
        })
    }

    pub fn type_of(&self) -> ExternalTypeof {
        self.0.type_of().into()
    }

    pub fn get_property(&self, vm: &mut ExternalVm, key: String) -> Result<JsValue, JsValue> {
        vm.with_scope(|scope| {
            let value = self.0.get_property(scope, key.into()).map_err(JsValue)?;
            Ok(JsValue(value))
        })
    }

    pub fn call(&self, vm: &mut ExternalVm, receiver: JsValue, args: js_sys::Array) -> Result<JsValue, JsValue> {
        vm.with_scope(|scope| {
            let args = args
                .iter()
                .map(|v| dash_value_from_wasm_value(scope, v))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| JsValue(DashValue::String(err.into())))?;

            self.0.apply(scope, receiver.0, args).map(JsValue).map_err(JsValue)
        })
    }
}
