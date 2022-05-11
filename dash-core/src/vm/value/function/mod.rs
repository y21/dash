use std::{
    any::Any,
    cell::RefCell,
    fmt::{self, Debug},
};

use crate::{
    gc::{handle::Handle, trace::Trace},
    vm::{dispatch::HandleResult, frame::Frame, local::LocalScope, Vm},
};

use self::{
    generator::{GeneratorFunction, GeneratorIterator},
    native::{CallContext, NativeFunction},
    user::UserFunction,
};

use super::{
    object::{NamedObject, Object, PropertyKey},
    Typeof, Value,
};

pub mod generator;
pub mod native;
pub mod user;

pub enum FunctionKind {
    Native(NativeFunction),
    User(UserFunction),
    Generator(GeneratorFunction),
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
}

impl fmt::Debug for FunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Native(_) => f.write_str("NativeFunction"),
            Self::User(_) => f.write_str("UserFunction"),
            Self::Generator(_) => f.write_str("GeneratorFunction"),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    name: Option<String>,
    kind: FunctionKind,
    obj: NamedObject,
    prototype: RefCell<Option<Handle<dyn Object>>>,
}

impl Function {
    pub fn new(vm: &mut Vm, name: Option<String>, kind: FunctionKind) -> Self {
        Self {
            name,
            kind,
            obj: NamedObject::new(vm),
            prototype: RefCell::new(None),
        }
    }

    pub fn with_obj(name: Option<String>, kind: FunctionKind, obj: NamedObject) -> Self {
        Self {
            name,
            kind,
            obj,
            prototype: RefCell::new(None),
        }
    }

    pub fn kind(&self) -> &FunctionKind {
        &self.kind
    }

    pub fn set_fn_prototype(&self, prototype: Handle<dyn Object>) {
        self.prototype.replace(Some(prototype));
    }
}

unsafe impl Trace for Function {
    fn trace(&self) {}
}

impl Object for Function {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        self.obj.get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
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
        match &self.kind {
            FunctionKind::Native(native) => {
                let cx = CallContext { args, scope, this };
                let result = native(cx);
                result
            }
            FunctionKind::User(uf) => {
                let sp = scope.stack.len();

                let argc = std::cmp::min(uf.params(), args.len());

                scope.stack.extend(args.into_iter().take(argc));

                let mut frame = Frame::from_function(uf, scope);
                frame.sp = sp;

                scope.vm.execute_frame(frame).map(|v| match v {
                    HandleResult::Return(v) => v,
                    HandleResult::Yield(_) => unreachable!(), // UserFunction cannot `yield`
                })
            }
            FunctionKind::Generator(gen) => {
                let iter = GeneratorIterator::new(callee, scope, args);
                Ok(scope.register(iter).into())
            }
        }
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
