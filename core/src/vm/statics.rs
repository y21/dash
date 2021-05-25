use std::{cell::RefCell, mem::MaybeUninit, rc::Rc};

use crate::js_std;

use super::value::function::NativeFunction;
use super::value::Value;

pub struct Statics {
    pub console_log: Rc<RefCell<Value>>,
    pub isnan: Rc<RefCell<Value>>,
    pub array_push: Rc<RefCell<Value>>,
    pub math_pow: Rc<RefCell<Value>>,
}

impl Statics {
    pub fn new() -> Self {
        Self {
            console_log: NativeFunction::new("log", js_std::console::log, None).into(),
            isnan: NativeFunction::new("isNaN", js_std::functions::is_nan, None).into(),
            array_push: NativeFunction::new("push", js_std::array::push, None).into(),
            math_pow: NativeFunction::new("pow", js_std::math::pow, None).into(),
        }
    }
}
