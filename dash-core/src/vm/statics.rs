use crate::gc::handle::Handle;
use crate::gc::Gc;
use crate::js_std;
use crate::vm::value::function::Function;
use crate::vm::value::function::FunctionKind;

use super::value::boxed::Boolean;
use super::value::boxed::Number;
use super::value::boxed::String;
use super::value::function::native::NativeFunction;
use super::value::object::NamedObject;
use super::value::object::Object;

use std::rc::Rc;

pub struct Statics {
    pub empty_str: Rc<str>,
    pub true_lit: Rc<str>,
    pub false_lit: Rc<str>,
    pub is_nan: Handle<dyn Object>,
    pub is_finite: Handle<dyn Object>,
    pub parse_float: Handle<dyn Object>,
    pub parse_int: Handle<dyn Object>,
    pub console: Handle<dyn Object>,
    pub console_log: Handle<dyn Object>,
    pub math: Handle<dyn Object>,
    pub math_floor: Handle<dyn Object>,
    pub math_abs: Handle<dyn Object>,
    pub math_acos: Handle<dyn Object>,
    pub math_acosh: Handle<dyn Object>,
    pub math_asin: Handle<dyn Object>,
    pub math_asinh: Handle<dyn Object>,
    pub math_atan: Handle<dyn Object>,
    pub math_atanh: Handle<dyn Object>,
    pub math_atan2: Handle<dyn Object>,
    pub math_cbrt: Handle<dyn Object>,
    pub math_ceil: Handle<dyn Object>,
    pub math_clz32: Handle<dyn Object>,
    pub math_cos: Handle<dyn Object>,
    pub math_cosh: Handle<dyn Object>,
    pub math_exp: Handle<dyn Object>,
    pub math_expm1: Handle<dyn Object>,
    pub math_log: Handle<dyn Object>,
    pub math_log1p: Handle<dyn Object>,
    pub math_log10: Handle<dyn Object>,
    pub math_log2: Handle<dyn Object>,
    pub math_round: Handle<dyn Object>,
    pub math_sin: Handle<dyn Object>,
    pub math_sinh: Handle<dyn Object>,
    pub math_sqrt: Handle<dyn Object>,
    pub math_tan: Handle<dyn Object>,
    pub math_tanh: Handle<dyn Object>,
    pub math_trunc: Handle<dyn Object>,
    pub object_ctor: Handle<dyn Object>,
    pub object_prototype: Handle<dyn Object>,
    pub number_ctor: Handle<dyn Object>,
    pub number_prototype: Handle<dyn Object>,
    pub number_tostring: Handle<dyn Object>,
    pub number_is_finite: Handle<dyn Object>,
    pub number_is_nan: Handle<dyn Object>,
    pub number_is_safe_integer: Handle<dyn Object>,
    pub number_to_fixed: Handle<dyn Object>,
    pub boolean_ctor: Handle<dyn Object>,
    pub boolean_tostring: Handle<dyn Object>,
    pub boolean_prototype: Handle<dyn Object>,
    pub boolean_valueof: Handle<dyn Object>,
    pub string_ctor: Handle<dyn Object>,
    pub string_prototype: Handle<dyn Object>,
    pub string_tostring: Handle<dyn Object>,
}

fn object(gc: &mut Gc<dyn Object>) -> Handle<dyn Object> {
    gc.register(NamedObject::null())
}

fn function(gc: &mut Gc<dyn Object>, name: &str, cb: NativeFunction) -> Handle<dyn Object> {
    let f = Function::with_obj(
        Some(name.into()),
        FunctionKind::Native(cb),
        NamedObject::null(),
    );
    gc.register(f)
}

impl Statics {
    pub fn new(gc: &mut Gc<dyn Object>) -> Self {
        let empty_str: Rc<str> = "".into();

        Self {
            true_lit: "true".into(),
            false_lit: "false".into(),
            empty_str: empty_str.clone(),
            console: object(gc),
            console_log: function(gc, "log", js_std::global::log),
            math: object(gc),
            math_floor: function(gc, "floor", js_std::math::floor),
            object_ctor: function(gc, "Object", js_std::object::constructor),
            object_prototype: object(gc),
            number_ctor: function(gc, "Number", js_std::number::constructor),
            number_prototype: gc.register(Number::with_obj(0.0, NamedObject::null())),
            number_tostring: function(gc, "toString", js_std::number::to_string),
            boolean_ctor: function(gc, "Boolean", js_std::boolean::constructor),
            boolean_tostring: function(gc, "toString", js_std::boolean::to_string),
            boolean_prototype: gc.register(Boolean::with_obj(false, NamedObject::null())),
            string_ctor: function(gc, "Boolean", js_std::string::constructor),
            string_prototype: gc.register(String::with_obj(empty_str, NamedObject::null())),
            is_nan: function(gc, "isNaN", js_std::global::is_nan),
            is_finite: function(gc, "isFinite", js_std::global::is_finite),
            parse_float: function(gc, "parseFloat", js_std::global::parse_float),
            parse_int: function(gc, "parseInt", js_std::global::parse_int),
            math_abs: function(gc, "abs", js_std::math::abs),
            math_acos: function(gc, "acos", js_std::math::acos),
            math_acosh: function(gc, "acosh", js_std::math::acosh),
            math_asin: function(gc, "asin", js_std::math::asin),
            math_asinh: function(gc, "asinh", js_std::math::asinh),
            math_atan: function(gc, "atan", js_std::math::atan),
            math_atanh: function(gc, "atanh", js_std::math::atanh),
            math_atan2: function(gc, "atan2", js_std::math::atan2),
            math_cbrt: function(gc, "cbrt", js_std::math::cbrt),
            math_ceil: function(gc, "ceil", js_std::math::ceil),
            math_clz32: function(gc, "clz32", js_std::math::clz32),
            math_cos: function(gc, "cos", js_std::math::cos),
            math_cosh: function(gc, "cosh", js_std::math::cosh),
            math_exp: function(gc, "exp", js_std::math::exp),
            math_expm1: function(gc, "expm1", js_std::math::expm1),
            math_log: function(gc, "log", js_std::math::log),
            math_log1p: function(gc, "log1p", js_std::math::log1p),
            math_log10: function(gc, "log10", js_std::math::log10),
            math_log2: function(gc, "log2", js_std::math::log2),
            math_round: function(gc, "round", js_std::math::round),
            math_sin: function(gc, "sin", js_std::math::sin),
            math_sinh: function(gc, "sinh", js_std::math::sinh),
            math_sqrt: function(gc, "sqrt", js_std::math::sqrt),
            math_tan: function(gc, "tan", js_std::math::tan),
            math_tanh: function(gc, "tanh", js_std::math::tanh),
            math_trunc: function(gc, "trunc", js_std::math::trunc),
            number_is_finite: function(gc, "isFinite", js_std::number::is_finite),
            number_is_nan: function(gc, "isNaN", js_std::number::is_nan),
            number_is_safe_integer: function(gc, "isSafeInteger", js_std::number::is_safe_integer),
            number_to_fixed: function(gc, "toFixed", js_std::number::to_fixed),
            boolean_valueof: function(gc, "valueOf", js_std::boolean::value_of),
            string_tostring: function(gc, "toString", js_std::string::to_string),
        }
    }

    pub fn get_true(&self) -> Rc<str> {
        self.true_lit.clone()
    }

    pub fn get_false(&self) -> Rc<str> {
        self.false_lit.clone()
    }

    pub fn empty_str(&self) -> Rc<str> {
        self.empty_str.clone()
    }
}
