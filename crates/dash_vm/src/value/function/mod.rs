use std::{
    any::Any,
    cell::RefCell,
    cmp::Ordering,
    fmt::{self, Debug},
    iter,
    rc::Rc,
};

use dash_proc_macro::Trace;

use crate::{
    dispatch::HandleResult,
    gc::{handle::Handle, trace::Trace},
    localscope::LocalScope,
    throw, Vm,
};

use self::{
    generator::GeneratorFunction,
    native::{CallContext, NativeFunction},
    r#async::AsyncFunction,
    user::UserFunction,
};

use super::{
    array::Array,
    object::{NamedObject, Object, PropertyKey, PropertyValue},
    Typeof, Unrooted, Value,
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

unsafe impl Trace for FunctionKind {
    fn trace(&self) {
        match self {
            Self::User(user) => user.trace(),
            Self::Generator(generator) => generator.trace(),
            Self::Async(async_) => async_.trace(),
            Self::Native(_) => {}
        }
    }
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

#[derive(Debug, Trace)]
pub struct Function {
    name: RefCell<Option<Rc<str>>>,
    kind: FunctionKind,
    obj: NamedObject,
    prototype: RefCell<Option<Handle<dyn Object>>>,
}

impl Function {
    pub fn new(vm: &Vm, name: Option<Rc<str>>, kind: FunctionKind) -> Self {
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

    pub fn get_fn_prototype(&self) -> Option<Handle<dyn Object>> {
        self.prototype.borrow().clone()
    }

    pub fn get_or_set_prototype(&self, scope: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        // can make this faster if we need to by directly accessing the prototype
        // without going through the property system
        let prototype = match self.get_property(scope, "prototype".into())? {
            Value::Undefined(_) => {
                let prototype = NamedObject::new(scope);
                scope.register(prototype)
            }
            Value::Object(o) => o,
            Value::External(o) => o.inner.clone(),
            _ => throw!(scope, TypeError, "prototype is not an object"),
        };

        Ok(prototype)
    }

    /// Creates a new instance of this function.
    pub fn new_instance(
        &self,
        this_handle: Handle<dyn Object>,
        scope: &mut LocalScope,
    ) -> Result<Handle<dyn Object>, Value> {
        let prototype = self.get_or_set_prototype(scope)?;
        let this = scope.register(NamedObject::with_prototype_and_constructor(prototype, this_handle));
        Ok(this)
    }
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
        FunctionKind::User(fun) => fun
            .handle_function_call(scope, this, args, is_constructor_call)
            .map(|v| match v {
                HandleResult::Return(v) => v,
                HandleResult::Yield(..) | HandleResult::Await(..) => unreachable!(), // UserFunction cannot `yield`/`await`
            }),
        FunctionKind::Async(fun) => fun.handle_function_call(scope, callee, this, args, is_constructor_call),
        FunctionKind::Generator(fun) => fun.handle_function_call(scope, callee, this, args, is_constructor_call),
    }
}

impl Object for Function {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        if let Some(key) = key.as_string() {
            match key {
                "name" => {
                    let name = self.name().unwrap_or_else(|| sc.statics.empty_str());
                    return Ok(Some(PropertyValue::static_default(Value::String(name))));
                }
                "prototype" => {
                    let mut prototype = self.prototype.borrow_mut();

                    let prototype = prototype.get_or_insert_with(|| {
                        let proto = NamedObject::new(sc);
                        sc.register(proto)
                    });
                    return Ok(Some(PropertyValue::static_default(Value::Object(prototype.clone()))));
                }
                _ => {}
            }
        }

        self.obj.get_own_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
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
        let this = self.new_instance(callee.clone(), scope)?;
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

pub(crate) fn adjust_stack_from_flat_call(
    scope: &mut LocalScope,
    user_function: &UserFunction,
    old_sp: usize,
    argc: usize,
) {
    // Conveniently, the arguments are all on the stack, in the order
    // we need it to be in, so we don't need to move anything there for that part.

    let expected_args = user_function.inner().params;

    // NB: Order is important, this needs to happen before pushing remaining
    // missing undefined values and truncating
    let rest = if user_function.inner().rest_local.is_some() {
        let args = scope
            .stack
            .drain(old_sp + expected_args..)
            .map(PropertyValue::static_default)
            .collect();

        let array = Array::from_vec(scope, args);
        let array = scope.register(array);
        Some(Value::Object(array))
    } else {
        None
    };

    match argc.cmp(&expected_args) {
        Ordering::Less => {
            scope
                .stack
                .extend(iter::repeat(Value::undefined()).take(expected_args - argc));
        }
        Ordering::Greater => {
            scope.stack.truncate(old_sp + expected_args);
        }
        _ => {}
    }

    scope.stack.extend(rest);
}

/// Extends the VM stack with provided arguments
fn extend_stack_from_args(args: Vec<Value>, expected_args: usize, scope: &mut LocalScope, is_rest: bool) {
    // Insert at most [param_count] amount of provided arguments on the stack
    // In the compiler we allocate local space for every parameter
    scope.stack.extend(args.iter().take(expected_args).cloned());

    // Insert undefined values for parameters without a value
    if expected_args > args.len() {
        scope
            .stack
            .extend(iter::repeat(Value::undefined()).take(expected_args - args.len()));
    }

    // Finally insert Value::Object([]) if this function uses the rest operator
    if is_rest {
        let args = args
            .get(expected_args..)
            .map(|s| s.iter().cloned().map(PropertyValue::static_default).collect())
            .unwrap_or_default();

        let array = Array::from_vec(scope, args);
        let array = scope.register(array);
        scope.stack.push(Value::Object(array));
    }
}
