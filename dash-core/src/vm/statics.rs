use crate::gc::handle::Handle;
use crate::gc::Gc;
use crate::js_std;
use crate::vm::value::function::Function;
use crate::vm::value::function::FunctionKind;

use super::value::boxed::Number;
use super::value::function::native::NativeFunction;
use super::value::object::NamedObject;
use super::value::object::Object;

use std::rc::Rc;

pub struct Statics {
    pub true_lit: Rc<str>,
    pub false_lit: Rc<str>,
    pub console: Handle<dyn Object>,
    pub math: Handle<dyn Object>,
    pub log: Handle<dyn Object>,
    pub floor: Handle<dyn Object>,
    pub object_ctor: Handle<dyn Object>,
    pub object_prototype: Handle<dyn Object>,
    pub number_ctor: Handle<dyn Object>,
    pub number_prototype: Handle<dyn Object>,
    pub number_tostring: Handle<dyn Object>,
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
        Self {
            true_lit: "true".into(),
            false_lit: "false".into(),
            console: object(gc),
            math: object(gc),
            log: function(gc, "log", js_std::global::log),
            floor: function(gc, "floor", js_std::math::floor),
            object_ctor: function(gc, "Object", js_std::object::constructor),
            object_prototype: object(gc),
            number_ctor: function(gc, "Number", js_std::number::constructor),
            number_prototype: gc.register(Number::with_obj(0.0, NamedObject::null())),
            number_tostring: function(gc, "toString", js_std::number::to_string),
        }
    }

    pub fn get_true(&self) -> Rc<str> {
        self.true_lit.clone()
    }

    pub fn get_false(&self) -> Rc<str> {
        self.false_lit.clone()
    }
}
