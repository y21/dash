use std::{
    any::Any,
    cell::RefCell,
    fmt::{self, Debug},
    rc::Rc,
};

use crate::{
    gc::{handle::Handle, trace::Trace},
    local::LocalScope,
    throw, Vm,
};

use self::{
    generator::GeneratorFunction,
    native::{CallContext, NativeFunction},
    r#async::AsyncFunction,
    user::UserFunction,
};

use super::{
    object::{NamedObject, Object, PropertyKey, PropertyValue},
    Typeof, Value,
};

pub mod r#async;
pub mod bound;
pub mod generator;
pub mod native;
pub mod user;
pub enum FunctionKind {
    Native(NativeFunction),
    User(UserFunction),
    Generator(GeneratorFunction),
    Async(AsyncFunction),
}

impl FunctionKind {
    pub fn as_native(&self) -> Option<&NativeFunction> {
        match self {
            Self::Native(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_user(&self) -> Option<&UserFunction> {
        match self {
            Self::User(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_generator(&self) -> Option<&GeneratorFunction> {
        match self {
            Self::Generator(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_async(&self) -> Option<&AsyncFunction> {
        match self {
            Self::Async(f) => Some(f),
            _ => None,
        }
    }
}

impl fmt::Debug for FunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Native(..) => f.write_str("NativeFunction"),
            Self::User(..) => f.write_str("UserFunction"),
            Self::Generator(..) => f.write_str("GeneratorFunction"),
            Self::Async(..) => f.write_str("AsyncFunction"),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    name: RefCell<Option<Rc<str>>>,
    kind: FunctionKind,
    obj: NamedObject,
    prototype: RefCell<Option<Handle<dyn Object>>>,
}

impl Function {
    pub fn new(vm: &mut Vm, name: Option<Rc<str>>, kind: FunctionKind) -> Self {
        let (proto, ctor) = (&vm.statics.function_proto, &vm.statics.function_ctor);

        Self::with_obj(
            name,
            kind,
            NamedObject::with_prototype_and_constructor(proto.clone(), ctor.clone()),
        )
    }

    pub fn with_obj(name: Option<Rc<str>>, kind: FunctionKind, obj: NamedObject) -> Self {
        Self {
            name: RefCell::new(name),
            kind,
            obj,
            prototype: RefCell::new(None),
        }
    }

    pub fn kind(&self) -> &FunctionKind {
        &self.kind
    }

    pub fn set_name(&self, name: Rc<str>) -> Option<Rc<str>> {
        self.name.borrow_mut().replace(name)
    }

    pub fn name(&self) -> Option<Rc<str>> {
        self.name.borrow().clone()
    }

    pub fn set_fn_prototype(&self, prototype: Handle<dyn Object>) {
        self.prototype.replace(Some(prototype));
    }
}

unsafe impl Trace for Function {
    fn trace(&self) {}
}

fn handle_call(
    fun: &Function,
    scope: &mut LocalScope,
    callee: Handle<dyn Object>,
    this: Value,
    args: Vec<Value>,
    is_constructor_call: bool,
) -> Result<Value, Value> {
    match &fun.kind {
        FunctionKind::Native(native) => {
            let cx = match is_constructor_call {
                true => CallContext::constructor(args, scope, this),
                false => CallContext::call(args, scope, this),
            };
            native(cx)
        }
        FunctionKind::User(fun) => fun.handle_function_call(scope, this, args, is_constructor_call),
        FunctionKind::Async(fun) => fun.handle_function_call(scope, this, args, is_constructor_call),
        FunctionKind::Generator(..) => {
            GeneratorFunction::handle_function_call(scope, callee, this, args, is_constructor_call)
        }
    }
}

impl Object for Function {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        if let Some(key) = key.as_string() {
            match key {
                "name" => {
                    let name = self.name().unwrap_or_else(|| sc.statics.empty_str());

                    return Ok(Value::String(name));
                }
                "prototype" => {
                    let mut prototype = self.prototype.borrow_mut();

                    let prototype = prototype.get_or_insert_with(|| {
                        let proto = NamedObject::new(sc);
                        sc.register(proto)
                    });
                    return Ok(Value::Object(prototype.clone()));
                }
                _ => {}
            }
        }

        self.obj.get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        self.obj.delete_property(sc, key)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        handle_call(self, scope, callee, this, args, false)
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        let prototype = match self.get_property(scope, "prototype".into())? {
            Value::Undefined(_) => {
                let prototype = NamedObject::new(scope);
                scope.register(prototype)
            }
            Value::Object(o) | Value::External(o) => o,
            _ => throw!(scope, "prototype is not an object"),
        };

        let this = scope.register(NamedObject::with_prototype_and_constructor(prototype, callee.clone()));

        handle_call(self, scope, callee, Value::Object(this), args, true)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(["length", "name"].iter().map(|&s| Value::String(s.into())).collect())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Function
    }
}
