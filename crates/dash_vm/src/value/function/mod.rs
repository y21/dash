use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::iter::{self};

use dash_proc_macro::Trace;

use crate::dispatch::HandleResult;
use crate::frame::This;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::arguments::Arguments;
use crate::{extract, Vm};
use dash_middle::interner::sym;

use self::r#async::AsyncFunction;
use self::closure::Closure;
use self::generator::GeneratorFunction;
use self::native::{CallContext, NativeFunction};
use self::user::UserFunction;

use super::array::Array;
use super::object::{NamedObject, Object, PropertyDataDescriptor, PropertyKey, PropertyValue, PropertyValueKind};
use super::ops::conversions::ValueConversion;
use super::string::JsString;
use super::{Root, Typeof, Unrooted, Value};

pub mod r#async;
pub mod bound;
pub mod closure;
pub mod generator;
pub mod native;
pub mod user;

pub enum FunctionKind {
    Native(NativeFunction),
    User(UserFunction),
    Generator(GeneratorFunction),
    Async(AsyncFunction),
    Closure(Closure),
}

unsafe impl Trace for FunctionKind {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            Self::User(user) => user.trace(cx),
            Self::Generator(generator) => generator.trace(cx),
            Self::Async(async_) => async_.trace(cx),
            Self::Native(_) => {}
            Self::Closure(user) => user.trace(cx),
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
            Self::Closure(..) => f.write_str("closure"),
        }
    }
}

#[derive(Debug, Trace)]
pub struct Function {
    name: RefCell<Option<JsString>>,
    kind: FunctionKind,
    obj: NamedObject,
    prototype: RefCell<Option<ObjectId>>,
}

impl Function {
    pub fn new(vm: &Vm, name: Option<JsString>, kind: FunctionKind) -> Self {
        Self::with_obj(
            name,
            kind,
            NamedObject::with_prototype_and_constructor(vm.statics.function_proto, vm.statics.function_ctor),
        )
    }

    pub fn with_obj(name: Option<JsString>, kind: FunctionKind, obj: NamedObject) -> Self {
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

    pub fn set_name(&self, name: JsString) -> Option<JsString> {
        self.name.borrow_mut().replace(name)
    }

    pub fn name(&self) -> Option<JsString> {
        *self.name.borrow()
    }

    pub fn set_fn_prototype(&self, prototype: ObjectId) {
        self.prototype.replace(Some(prototype));
    }

    pub fn get_fn_prototype(&self) -> Option<ObjectId> {
        *self.prototype.borrow()
    }

    pub fn get_or_set_prototype(&self, scope: &mut LocalScope) -> ObjectId {
        *self.prototype.borrow_mut().get_or_insert_with(|| {
            let proto = NamedObject::new(scope);
            scope.register(proto)
        })
    }

    /// Creates a new instance of this function.
    pub fn new_instance(&self, this_handle: ObjectId, scope: &mut LocalScope) -> Result<ObjectId, Value> {
        let prototype = self.get_or_set_prototype(scope);
        let this = scope.register(NamedObject::with_prototype_and_constructor(prototype, this_handle));
        Ok(this)
    }

    pub fn inner_user_function(&self) -> Option<&UserFunction> {
        match &self.kind {
            FunctionKind::User(function) => Some(function),
            FunctionKind::Generator(generator) => Some(&generator.function),
            FunctionKind::Async(function) => Some(&function.inner.function),
            FunctionKind::Closure(closure) => Some(&closure.fun),
            FunctionKind::Native(_) => None,
        }
    }
}

fn handle_call(
    fun: &Function,
    scope: &mut LocalScope,
    callee: ObjectId,
    this: This,
    args: Vec<Value>,
    is_constructor_call: bool,
) -> Result<Unrooted, Unrooted> {
    match &fun.kind {
        FunctionKind::Native(native) => {
            let this = this.to_value(scope)?;
            // TODO: pass `This` to native fns as-is?
            let cx = match is_constructor_call {
                true => CallContext::constructor(args, scope, this),
                false => CallContext::call(args, scope, this),
            };
            match native(cx) {
                Ok(v) => Ok(v.into()),
                Err(v) => Err(v.into()),
            }
        }
        FunctionKind::User(fun) => fun
            .handle_function_call(scope, this, args, is_constructor_call)
            .map(|v| match v {
                HandleResult::Return(v) => v,
                HandleResult::Yield(..) | HandleResult::Await(..) => unreachable!(), // UserFunction cannot `yield`/`await`
            })
            .map_err(Into::into),
        FunctionKind::Async(fun) => fun
            .handle_function_call(scope, callee, this, args, is_constructor_call)
            .map(Into::into),
        FunctionKind::Generator(fun) => fun
            .handle_function_call(scope, callee, this, args, is_constructor_call)
            .map(Into::into),
        FunctionKind::Closure(fun) => fun.handle_function_call(scope, this, args, is_constructor_call),
    }
}

impl Object for Function {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        if let Some(key) = key.as_string() {
            match key.sym() {
                sym::name => {
                    let name = self.name().unwrap_or_else(|| sym::empty.into());
                    return Ok(Some(PropertyValue {
                        kind: PropertyValueKind::Static(Value::string(name)),
                        descriptor: PropertyDataDescriptor::CONFIGURABLE,
                    }));
                }
                sym::length => {
                    if let Some(function) = self.inner_user_function() {
                        return Ok(Some(PropertyValue {
                            kind: PropertyValueKind::Static(Value::number(function.inner().params as f64)),
                            descriptor: PropertyDataDescriptor::CONFIGURABLE,
                        }));
                    }
                }
                sym::prototype => {
                    let prototype = self.get_or_set_prototype(sc);
                    return Ok(Some(PropertyValue::static_empty(Value::object(prototype))));
                }
                _ => {}
            }
        }

        self.obj.get_own_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        if let Some(sym::prototype) = key.as_string().map(JsString::sym) {
            let prototype = value.get_or_apply(sc, This::Default).root(sc)?;
            // TODO: function prototype does not need to be an object
            *self.prototype.borrow_mut() = Some(prototype.to_object(sc)?);
            return Ok(());
        }

        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        self.obj.delete_property(sc, key)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: This,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        handle_call(self, scope, callee, this, args, false)
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        _this: This,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        let this = 'this: {
            if let Some(user) = self.inner_user_function() {
                if user.inner().has_extends_clause {
                    // We don't immediately create an instance when instantiating a subclass.
                    // The super() call desugaring will initialize `this`

                    break 'this This::BeforeSuper;
                }
            }

            let inst = self.new_instance(callee, scope)?;
            This::Bound(Value::object(inst))
        };

        handle_call(self, scope, callee, this, args, true)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(vec![Value::string(sym::length.into()), Value::string(sym::name.into())])
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Function
    }

    extract!(self);
}

/// Returns the `arguments` object, iff the function needs it.
pub(crate) fn adjust_stack_from_flat_call(
    scope: &mut LocalScope,
    user_function: &UserFunction,
    old_sp: usize,
    argc: usize,
) -> Option<ObjectId> {
    let mut arguments = None;
    if user_function.inner().references_arguments {
        let args = scope.stack[old_sp..].to_vec();
        // TODO: this assertion is wrong for (function(){ return arguments })(...[1, 2]). args.len() is correct
        debug_assert_eq!(args.len(), argc);
        let args = Arguments::new(scope, args);
        let args = scope.register(args);
        arguments = Some(args);
    }

    // Conveniently, the arguments are all on the stack, in the order
    // we need it to be in, so we don't need to move anything there for that part.

    let expected_args = user_function.inner().params;

    // NB: Order is important, this needs to happen before pushing remaining
    // missing undefined values and truncating
    let rest = if user_function.inner().rest_local.is_some() {
        let stack_len = scope.stack.len();
        let args = scope
            .stack
            .drain((old_sp + expected_args).min(stack_len)..)
            .map(PropertyValue::static_default)
            .collect();

        let array = Array::from_vec(scope, args);
        let array = scope.register(array);
        Some(Value::object(array))
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
    arguments
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
        scope.stack.push(Value::object(array));
    }
}
