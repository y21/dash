use std::{cell::RefCell, rc::Rc};

use crate::js_std;

use super::value::function::NativeFunction;
use super::value::Value;

pub struct Statics {
    pub console_log: Rc<RefCell<Value>>,
    pub isnan: Rc<RefCell<Value>>,
    pub array_push: Rc<RefCell<Value>>,
    pub math_pow: Rc<RefCell<Value>>,
    pub math_abs: Rc<RefCell<Value>>,
    pub math_ceil: Rc<RefCell<Value>>,
    pub math_floor: Rc<RefCell<Value>>,
    pub math_max: Rc<RefCell<Value>>,
    pub object_define_property: Rc<RefCell<Value>>,
    pub object_get_own_property_names: Rc<RefCell<Value>>,
    pub error_ctor: Rc<RefCell<Value>>,
}

impl Statics {
    pub fn new() -> Self {
        Self {
            console_log: NativeFunction::new("log", js_std::console::log, None, false).into(),
            isnan: NativeFunction::new("isNaN", js_std::functions::is_nan, None, false).into(),
            array_push: NativeFunction::new("push", js_std::array::push, None, false).into(),
            math_pow: NativeFunction::new("pow", js_std::math::pow, None, false).into(),
            math_abs: NativeFunction::new("abs", js_std::math::abs, None, false).into(),
            math_ceil: NativeFunction::new("ceil", js_std::math::ceil, None, false).into(),
            math_floor: NativeFunction::new("floor", js_std::math::floor, None, false).into(),
            math_max: NativeFunction::new("max", js_std::math::max, None, false).into(),
            object_define_property: NativeFunction::new(
                "defineProperty",
                js_std::object::define_property,
                None,
                false,
            )
            .into(),
            object_get_own_property_names: NativeFunction::new(
                "getOwnPropertyNames",
                js_std::object::get_own_property_names,
                None,
                false,
            )
            .into(),
            error_ctor: NativeFunction::new("Error", js_std::error::error_constructor, None, true)
                .into(),
        }
    }
}
