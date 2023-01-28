pub mod array;
pub mod arraybuffer;
pub mod boxed;
pub mod conversions;
pub mod error;
pub mod function;
pub mod inspect;
pub mod map;
pub mod object;
pub mod ops;
pub mod primitive;
pub mod promise;
pub mod regex;
pub mod set;
pub mod typedarray;

use std::rc::Rc;

use dash_middle::compiler::{constant::Constant, external::External};
use dash_middle::parser::statement::FunctionKind as ParserFunctionKind;
use dash_middle::util::ThreadSafeStorage;
use dash_proc_macro::Trace;

use crate::{delegate, throw};
use crate::{
    gc::{handle::Handle, trace::Trace},
    value::{
        function::FunctionKind,
        primitive::{Null, Undefined},
    },
};

use self::function::r#async::AsyncFunction;
use self::object::PropertyValue;
use self::primitive::{Number, PrimitiveCapabilities};
use self::regex::RegExp;
use self::{
    function::{generator::GeneratorFunction, user::UserFunction, Function},
    object::{Object, PropertyKey},
    primitive::Symbol,
};

use super::{local::LocalScope, Vm};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Value {
    /// The number type
    Number(Number),
    /// The boolean type
    Boolean(bool),
    /// The string type
    String(Rc<str>),
    /// The undefined type
    Undefined(Undefined),
    /// The null type
    Null(Null),
    /// The symbol type
    Symbol(Symbol),
    /// The object type
    Object(Handle<dyn Object>),
    /// An "external" value that is being used by other functions.
    External(Handle<dyn Object>),
}

unsafe impl Trace for Value {
    fn trace(&self) {
        if let Value::External(handle) | Value::Object(handle) = self {
            handle.trace();
        }
    }
}

impl Value {
    pub fn from_constant(constant: Constant, vm: &mut Vm) -> Self {
        match constant {
            Constant::Number(n) => Value::number(n),
            Constant::Boolean(b) => Value::Boolean(b),
            Constant::String(s) => Value::String(s),
            Constant::Undefined => Value::undefined(),
            Constant::Null => Value::null(),
            Constant::Regex(nodes, source) => {
                let regex = RegExp::new(nodes, source, vm);
                Value::Object(vm.register(regex))
            }
            Constant::Function(f) => {
                let mut externals = Vec::new();

                for External { id, is_external } in f.externals.iter().copied() {
                    let id = usize::from(id);

                    let val = if is_external {
                        Value::External(vm.get_external(id).expect("Referenced local not found").clone())
                    } else {
                        vm.get_local(id).expect("Referenced local not found")
                    };

                    fn register<O: Object + 'static>(vm: &mut Vm, idx: usize, o: O) -> Handle<dyn Object> {
                        let handle = vm.gc.register(o);
                        vm.set_local(idx, Value::External(handle.clone()));
                        handle
                    }

                    let obj = match val {
                        Value::Number(n) => register(vm, id, n),
                        Value::Boolean(b) => register(vm, id, b),
                        Value::String(s) => register(vm, id, s),
                        Value::Undefined(u) => register(vm, id, u),
                        Value::Null(n) => register(vm, id, n),
                        Value::Symbol(s) => register(vm, id, s),
                        Value::External(e) => e,
                        Value::Object(o) => {
                            vm.set_local(id, Value::External(o.clone()));
                            o
                        }
                    };

                    externals.push(obj);
                }

                let name: Option<Rc<str>> = f.name.as_deref().map(Into::into);
                let ty = f.ty;
                let is_async = f.r#async;

                let fun = UserFunction::new(f, externals.into());

                let kind = match ty {
                    ParserFunctionKind::Function | ParserFunctionKind::Arrow => {
                        if is_async {
                            FunctionKind::Async(AsyncFunction::new(fun))
                        } else {
                            FunctionKind::User(fun)
                        }
                    }
                    ParserFunctionKind::Generator => FunctionKind::Generator(GeneratorFunction::new(fun)),
                };

                let function = Function::new(vm, name, kind);
                vm.gc.register(function).into()
            }
            Constant::Identifier(_) => unreachable!(),
        }
    }

    pub fn set_property(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey<'static>,
        value: PropertyValue,
    ) -> Result<(), Value> {
        match self {
            Self::Object(h) => h.set_property(sc, key, value),
            Self::Number(n) => n.set_property(sc, key, value),
            Self::Boolean(b) => b.set_property(sc, key, value),
            Self::String(s) => s.set_property(sc, key, value),
            Self::External(h) => h.set_property(sc, key, value),
            Self::Undefined(u) => u.set_property(sc, key, value),
            Self::Null(n) => n.set_property(sc, key, value),
            Self::Symbol(s) => s.set_property(sc, key, value),
        }
    }

    pub fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        match self {
            Self::Object(o) => o.get_property(sc, key),
            Self::Number(n) => n.get_property(sc, key),
            Self::Boolean(b) => b.get_property(sc, key),
            Self::String(s) => s.get_property(sc, key),
            Self::External(o) => o.get_property(sc, key),
            Self::Undefined(u) => u.get_property(sc, key),
            Self::Null(n) => n.get_property(sc, key),
            Self::Symbol(s) => s.get_property(sc, key),
        }
    }

    pub fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        match self {
            Self::Object(o) => o.delete_property(sc, key),
            Self::Number(n) => n.delete_property(sc, key),
            Self::Boolean(b) => b.delete_property(sc, key),
            Self::String(s) => s.delete_property(sc, key),
            Self::External(o) => o.delete_property(sc, key),
            Self::Undefined(u) => u.delete_property(sc, key),
            Self::Null(n) => n.delete_property(sc, key),
            Self::Symbol(s) => s.delete_property(sc, key),
        }
    }

    pub fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        match self {
            Self::Object(o) => o.apply(sc, this, args),
            Self::External(o) => o.apply(sc, this, args),
            Self::Number(n) => throw!(sc, "{} is not a function", n),
            Self::Boolean(b) => throw!(sc, "{} is not a function", b),
            Self::String(s) => throw!(sc, "{} is not a function", s),
            Self::Undefined(_) => throw!(sc, "undefined is not a function"),
            Self::Null(_) => throw!(sc, "null is not a function"),
            Self::Symbol(s) => throw!(sc, "{:?} is not a function", s),
        }
    }

    pub fn construct(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Value, Value> {
        match self {
            Self::Object(o) => o.construct(sc, this, args),
            Self::External(o) => o.construct(sc, this, args),
            Self::Number(n) => throw!(sc, "{} is not a constructor", n),
            Self::Boolean(b) => throw!(sc, "{} is not a constructor", b),
            Self::String(s) => throw!(sc, "{} is not a constructor", s),
            Self::Undefined(_) => throw!(sc, "undefined is not a constructor"),
            Self::Null(_) => throw!(sc, "null is not a constructor"),
            Self::Symbol(s) => throw!(sc, "{:?} is not a constructor", s),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Number(Number(n)) => *n != 0.0 && !n.is_nan(),
            Value::Symbol(_) => true,
            Value::Object(_) => true,
            Value::Undefined(_) => false,
            Value::Null(_) => false,
            Value::External(_) => todo!(),
        }
    }

    pub fn is_nullish(&self) -> bool {
        match self {
            Value::Null(_) => true,
            Value::Undefined(_) => true,
            Value::External(_) => todo!(),
            _ => false,
        }
    }

    pub fn undefined() -> Value {
        Value::Undefined(Undefined)
    }

    pub fn null() -> Value {
        Value::Null(Null)
    }

    pub fn number(n: f64) -> Value {
        Value::Number(Number(n))
    }

    pub fn unbox_external(self) -> Value {
        match self {
            Value::Boolean(b) => b.unbox(),
            Value::String(s) => s.unbox(),
            Value::Number(n) => n.unbox(),
            Value::Symbol(s) => s.unbox(),
            Value::Object(o) => Value::Object(o),
            Value::Undefined(u) => u.unbox(),
            Value::Null(n) => n.unbox(),
            Value::External(ext) => ext
                .as_primitive_capable()
                .map(|p| p.unbox())
                .unwrap_or_else(|| Value::Object(ext)),
        }
    }

    /// Boxes this value
    ///
    /// If this value already is an object, then this will wrap it in a new allocation
    pub fn into_boxed(self) -> Box<dyn Object> {
        match self {
            Value::Boolean(b) => Box::new(b),
            Value::Number(n) => Box::new(n),
            Value::String(s) => Box::new(s),
            Value::Null(n) => Box::new(n),
            Value::Undefined(u) => Box::new(u),
            Value::Object(o) => Box::new(o),
            Value::External(o) => Box::new(o), // TODO: is this correct?
            Value::Symbol(s) => Box::new(s),
        }
    }

    pub fn into_option(self) -> Option<Self> {
        match self {
            Value::Undefined(_) => None,
            _ => Some(self),
        }
    }

    pub fn type_of(&self) -> Typeof {
        match self {
            Self::Boolean(_) => Typeof::Boolean,
            Self::External(e) => e.type_of(),
            Self::Number(_) => Typeof::Number,
            Self::String(_) => Typeof::String,
            Self::Undefined(_) => Typeof::Undefined,
            Self::Object(o) => o.type_of(),
            Self::Null(_) => Typeof::Object,
            Self::Symbol(_) => Typeof::Symbol,
        }
    }

    pub fn instanceof(&self, ctor: &Self, sc: &mut LocalScope) -> Result<bool, Value> {
        let obj = match self {
            Self::Object(obj) | Self::External(obj) => obj,
            _ => return Ok(false),
        };

        // TODO: repeat for all objects in prototype chain
        let target_proto = ctor.get_property(sc, "prototype".into())?;
        let this_proto = obj.get_prototype(sc)?;

        Ok(this_proto == target_proto)
    }
}

#[derive(Debug)]
pub enum Typeof {
    Undefined,
    Object,
    Boolean,
    Number,
    Bigint,
    String,
    Symbol,
    Function,
}

impl Typeof {
    pub fn as_value(&self) -> Value {
        match self {
            Self::Undefined => Value::String("undefined".into()),
            Self::Object => Value::String("object".into()),
            Self::Boolean => Value::String("boolean".into()),
            Self::Number => Value::String("number".into()),
            Self::Bigint => Value::String("bigint".into()),
            Self::String => Value::String("string".into()),
            Self::Symbol => Value::String("symbol".into()),
            Self::Function => Value::String("function".into()),
        }
    }
}

pub trait ValueContext {
    fn unwrap_or_undefined(self) -> Value;
    fn unwrap_or_null(self) -> Value;
    fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value>;
}

impl ValueContext for Option<Value> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Some(x) => x,
            None => Value::undefined(),
        }
    }

    fn unwrap_or_null(self) -> Value {
        match self {
            Some(x) => x,
            None => Value::null(),
        }
    }

    fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value> {
        match self {
            Some(x) => Ok(x),
            None => throw!(vm, message),
        }
    }
}

impl ValueContext for Option<&Value> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Some(x) => x.clone(), // Values are cheap to clone
            None => Value::undefined(),
        }
    }

    fn unwrap_or_null(self) -> Value {
        match self {
            Some(x) => x.clone(),
            None => Value::null(),
        }
    }

    fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value> {
        match self {
            Some(x) => Ok(x.clone()),
            None => throw!(vm, message),
        }
    }
}

impl<E> ValueContext for Result<Value, E> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Ok(x) => x,
            Err(_) => Value::undefined(),
        }
    }

    fn unwrap_or_null(self) -> Value {
        match self {
            Ok(x) => x,
            Err(_) => Value::null(),
        }
    }

    fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value> {
        match self {
            Ok(x) => Ok(x),
            Err(_) => throw!(vm, message),
        }
    }
}

pub type ThreadSafeValue = ThreadSafeStorage<Value>;

/// A wrapper type for builtin objects that are protected from poisoning
///
/// The compiler can sometimes emit specialized opcodes for builtin functions,
/// such as Math.clz32 when called with a number, which skips all the property lookups.
/// As soon as builtins are mutated, this optimization is "unsafe", as the change
/// can be observed, for example:
/// ```js
/// Math.clz32 = () => 0;
/// assert(Math.clz32(1 << 30), 1);
/// ```
/// The compiler emits a clz32 opcode here, which would ignore the trapped function,
/// so the assert would fail here.
///
/// For this reason we wrap builtins in a `PureBuiltin`, which, when mutated, will
/// set a VM flag that makes the specialized opcodes fall back to the slow path (property lookup).
#[derive(Debug, Clone, Trace)]
pub struct PureBuiltin<O: Object> {
    inner: O,
}

impl<O: Object> PureBuiltin<O> {
    pub fn new(inner: O) -> Self {
        Self { inner }
    }
}

impl<O: Object + 'static> Object for PureBuiltin<O> {
    delegate!(
        inner,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        get_prototype,
        apply,
        construct,
        type_of
    );

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        sc.impure_builtins();
        self.inner.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        sc.impure_builtins();
        self.inner.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        sc.impure_builtins();
        self.inner.set_prototype(sc, value)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        &self.inner
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.inner.own_keys()
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        self.inner.as_primitive_capable()
    }
}
