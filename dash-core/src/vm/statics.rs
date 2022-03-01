use crate::gc::handle::Handle;
use crate::gc::Gc;
use crate::js_std;
use crate::vm::value::function::Function;
use crate::vm::value::function::FunctionKind;

use super::value::function::native::NativeFunction;
use super::value::object::AnonymousObject;
use super::value::object::Object;

pub struct Statics {
    pub console: Handle<dyn Object>,
    pub math: Handle<dyn Object>,
    pub log: Handle<dyn Object>,
    pub floor: Handle<dyn Object>,
}

fn object(gc: &mut Gc<dyn Object>) -> Handle<dyn Object> {
    gc.register(AnonymousObject::new())
}

fn function(gc: &mut Gc<dyn Object>, name: &str, cb: NativeFunction) -> Handle<dyn Object> {
    let f = Function::new(name.into(), FunctionKind::Native(cb));
    gc.register(f)
}

impl Statics {
    pub fn new(gc: &mut Gc<dyn Object>) -> Self {
        Self {
            console: object(gc),
            math: object(gc),
            log: function(gc, "log", js_std::global::log),
            floor: function(gc, "floor", js_std::math::floor),
        }
    }
}
