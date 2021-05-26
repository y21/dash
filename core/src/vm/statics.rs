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
    pub weakset_ctor: Rc<RefCell<Value>>,
    pub weakset_has: Rc<RefCell<Value>>,
    pub weakset_add: Rc<RefCell<Value>>,
    pub weakset_delete: Rc<RefCell<Value>>,
}

macro_rules! register_glob_method {
    ($name:expr, $path:expr) => {
        Value::from(NativeFunction::new($name, $path, None, false)).into()
    };
}

macro_rules! register_ctor {
    ($name:expr, $path:expr) => {
        Value::from(NativeFunction::new($name, $path, None, true)).into()
    };
}

impl Statics {
    pub fn new() -> Self {
        Self {
            console_log: register_glob_method!("log", js_std::console::log),
            isnan: register_glob_method!("isNaN", js_std::functions::is_nan),
            array_push: register_glob_method!("push", js_std::array::push),

            math_pow: register_glob_method!("pow", js_std::math::pow),
            math_abs: register_glob_method!("abs", js_std::math::abs),
            math_ceil: register_glob_method!("ceil", js_std::math::ceil),
            math_floor: register_glob_method!("floor", js_std::math::floor),
            math_max: register_glob_method!("max", js_std::math::max),
            object_define_property: register_glob_method!(
                "defineProperty",
                js_std::object::define_property
            ),
            object_get_own_property_names: register_glob_method!(
                "getOwnPropertyNames",
                js_std::object::get_own_property_names
            ),
            error_ctor: register_ctor!("Error", js_std::error::error_constructor),
            weakset_ctor: register_ctor!("WeakSet", js_std::weakset::weakset_constructor),
            weakset_has: register_glob_method!("has", js_std::weakset::has),
            weakset_add: register_glob_method!("add", js_std::weakset::add),
            weakset_delete: register_glob_method!("delete", js_std::weakset::delete),
        }
    }
}
