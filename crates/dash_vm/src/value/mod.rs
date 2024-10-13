pub mod arguments;
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

use std::ops::ControlFlow;

use dash_middle::interner;
use dash_middle::util::ThreadSafeStorage;
use dash_proc_macro::Trace;

pub mod string;
use crate::gc::interner::sym;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::gc::ObjectId;
use crate::util::cold_path;
use crate::value::primitive::{Null, Undefined};
use crate::{delegate, throw, Vm};

use self::object::{Object, PropertyKey, PropertyValue};
use self::primitive::{InternalSlots, Number, Symbol};
use self::string::JsString;
use super::localscope::LocalScope;

/// A packed JavaScript value.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Value(u64);

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Value [packed]").field(&self.0).finish()
    }
}

#[expect(clippy::unusual_byte_groupings)]
impl Value {
    const TAG_MASK: u64 = 0b1_11111111111_111 << (64 - 15);

    const INFINITY_MASK: u64 = 0b0_11111111111_000 << (64 - 15);
    const NEG_INFINITY_MASK: u64 = 0b1_11111111111_000 << (64 - 15);
    const NAN_MASK: u64 = 0b0_11111111111_100 << (64 - 15);

    const BOOLEAN_MASK: u64 = 0b0_11111111111_001 << (64 - 15);
    const STRING_MASK: u64 = 0b0_11111111111_010 << (64 - 15);
    const UNDEFINED_MASK: u64 = 0b0_11111111111_011 << (64 - 15);
    const NULL_MASK: u64 = 0b1_11111111111_001 << (64 - 15);
    const SYMBOL_MASK: u64 = 0b1_11111111111_010 << (64 - 15);
    const OBJECT_MASK: u64 = 0b1_11111111111_011 << (64 - 15);
    const EXTERNAL_MASK: u64 = 0b1_11111111111_101 << (64 - 15);

    pub fn number(v: f64) -> Self {
        // TODO: masking etc. maybe? def. need to be careful here if the bitpattern is untrusted...
        let num = Self(v.to_bits());
        assert!(matches!(num.unpack(), ValueKind::Number(_)));
        num
    }

    pub fn raw(self) -> u64 {
        self.0
    }

    pub fn boolean(b: bool) -> Self {
        Self(Self::BOOLEAN_MASK | b as u64)
    }

    pub fn undefined() -> Self {
        Self(Self::UNDEFINED_MASK)
    }

    pub fn null() -> Self {
        Self(Self::NULL_MASK)
    }

    pub fn symbol(v: Symbol) -> Self {
        Self(Self::SYMBOL_MASK | v.sym().raw() as u64)
    }

    pub fn string(v: JsString) -> Self {
        Self(Self::STRING_MASK | v.sym().raw() as u64)
    }

    pub fn object(id: ObjectId) -> Self {
        Self(Self::OBJECT_MASK | id.raw() as u64)
    }

    pub fn external(id: ObjectId) -> Self {
        Self(Self::EXTERNAL_MASK | id.raw() as u64)
    }
}

pub trait Unpack {
    type Output;
    fn unpack(self) -> Self::Output;
}
impl Unpack for Value {
    type Output = ValueKind;

    /// Unpacks the value so it can be matched
    fn unpack(self) -> Self::Output {
        // TODO: find out why codegen is bad
        #[expect(clippy::wildcard_in_or_patterns)]
        match self.0 & Self::TAG_MASK {
            Self::BOOLEAN_MASK => ValueKind::Boolean(self.0 as u8 == 1),
            Self::STRING_MASK => ValueKind::String(JsString::from(interner::Symbol::from_raw(self.0 as u32))),
            Self::UNDEFINED_MASK => ValueKind::Undefined(Undefined),
            Self::NULL_MASK => ValueKind::Null(Null),
            Self::SYMBOL_MASK => {
                ValueKind::Symbol(Symbol::new(JsString::from(interner::Symbol::from_raw(self.0 as u32))))
            }
            Self::OBJECT_MASK => ValueKind::Object(ObjectId::from_raw(self.0 as u32)),
            Self::EXTERNAL_MASK => ValueKind::External(ExternalValue {
                inner: ObjectId::from_raw(self.0 as u32),
            }),
            Self::INFINITY_MASK | Self::NEG_INFINITY_MASK | Self::NAN_MASK | _ => {
                // Anything else is a double.
                ValueKind::Number(Number(f64::from_bits(self.0)))
            }
        }
    }
}

impl Unpack for &Value {
    type Output = ValueKind;
    fn unpack(self) -> Self::Output {
        (*self).unpack()
    }
}

impl<U: Unpack> Unpack for Option<U> {
    type Output = Option<U::Output>;
    fn unpack(self) -> Self::Output {
        self.map(Unpack::unpack)
    }
}

unsafe impl Trace for Value {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self.unpack() {
            ValueKind::Object(o) => o.trace(cx),
            ValueKind::External(e) => e.trace(cx),
            ValueKind::String(s) => s.trace(cx),
            ValueKind::Number(_) => {}
            ValueKind::Boolean(_) => {}
            ValueKind::Undefined(_) => {}
            ValueKind::Null(_) => {}
            ValueKind::Symbol(s) => s.trace(cx),
        }
    }
}

impl Object for Value {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        match self.unpack() {
            ValueKind::Number(n) => n.get_own_property_descriptor(sc, key),
            ValueKind::Boolean(b) => b.get_own_property_descriptor(sc, key),
            ValueKind::String(s) => s.get_own_property_descriptor(sc, key),
            ValueKind::Undefined(u) => u.get_own_property_descriptor(sc, key),
            ValueKind::Null(n) => n.get_own_property_descriptor(sc, key),
            ValueKind::Symbol(s) => s.get_own_property_descriptor(sc, key),
            ValueKind::Object(o) => o.get_own_property_descriptor(sc, key),
            ValueKind::External(e) => e.get_own_property_descriptor(sc, key),
        }
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        match self.unpack() {
            ValueKind::Object(h) => h.set_property(sc, key, value),
            ValueKind::Number(n) => n.set_property(sc, key, value),
            ValueKind::Boolean(b) => b.set_property(sc, key, value),
            ValueKind::String(s) => s.set_property(sc, key, value),
            ValueKind::External(h) => h.set_property(sc, key, value),
            ValueKind::Undefined(u) => u.set_property(sc, key, value),
            ValueKind::Null(n) => n.set_property(sc, key, value),
            ValueKind::Symbol(s) => s.set_property(sc, key, value),
        }
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        match self.unpack() {
            ValueKind::Object(o) => o.delete_property(sc, key),
            ValueKind::Number(n) => n.delete_property(sc, key),
            ValueKind::Boolean(b) => b.delete_property(sc, key),
            ValueKind::String(s) => s.delete_property(sc, key),
            ValueKind::External(o) => o.delete_property(sc, key),
            ValueKind::Undefined(u) => u.delete_property(sc, key),
            ValueKind::Null(n) => n.delete_property(sc, key),
            ValueKind::Symbol(s) => s.delete_property(sc, key),
        }
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        match self.unpack() {
            ValueKind::Number(n) => n.set_prototype(sc, value),
            ValueKind::Boolean(b) => b.set_prototype(sc, value),
            ValueKind::String(s) => s.set_prototype(sc, value),
            ValueKind::Undefined(u) => u.set_prototype(sc, value),
            ValueKind::Null(n) => n.set_prototype(sc, value),
            ValueKind::Symbol(s) => s.set_prototype(sc, value),
            ValueKind::Object(o) => o.set_prototype(sc, value),
            ValueKind::External(e) => e.set_prototype(sc, value),
        }
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        match self.unpack() {
            ValueKind::Number(n) => n.get_prototype(sc),
            ValueKind::Boolean(b) => b.get_prototype(sc),
            ValueKind::String(s) => s.get_prototype(sc),
            ValueKind::Undefined(u) => u.get_prototype(sc),
            ValueKind::Null(n) => n.get_prototype(sc),
            ValueKind::Symbol(s) => s.get_prototype(sc),
            ValueKind::Object(o) => o.get_prototype(sc),
            ValueKind::External(e) => e.get_prototype(sc),
        }
    }

    fn apply(&self, scope: &mut LocalScope, _: ObjectId, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        // self.apply(scope, this, args)
        todo!()
    }

    fn as_any(&self, _: &Vm) -> &dyn std::any::Any {
        // self
        todo!()
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        match self.unpack() {
            ValueKind::Number(n) => n.own_keys(sc),
            ValueKind::Boolean(b) => b.own_keys(sc),
            ValueKind::String(s) => s.own_keys(sc),
            ValueKind::Undefined(u) => u.own_keys(sc),
            ValueKind::Null(n) => n.own_keys(sc),
            ValueKind::Symbol(s) => s.own_keys(sc),
            ValueKind::Object(o) => o.own_keys(sc),
            ValueKind::External(e) => e.own_keys(sc),
        }
    }

    fn type_of(&self, vm: &Vm) -> Typeof {
        match self.unpack() {
            ValueKind::Boolean(_) => Typeof::Boolean,
            ValueKind::External(e) => e.type_of(vm),
            ValueKind::Number(_) => Typeof::Number,
            ValueKind::String(_) => Typeof::String,
            ValueKind::Undefined(_) => Typeof::Undefined,
            ValueKind::Object(o) => o.type_of(vm),
            ValueKind::Null(_) => Typeof::Object,
            ValueKind::Symbol(_) => Typeof::Symbol,
        }
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        _: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.construct(scope, this, args)
    }

    fn internal_slots(&self, vm: &Vm) -> Option<&dyn InternalSlots> {
        todo!("???") // how do we implement this? we cant return the unpacked &f64
        // Idea: return Some(self) and delegate to the inner ones in dyn Internalslots
        // match self.unpack() {
        //     ValueKind::Number(n) => Some(n),
        //     ValueKind::Boolean(b) => Some(b),
        //     ValueKind::String(s) => Some(s),
        //     ValueKind::Undefined(_) | Value::Null(_) => None,
        //     ValueKind::Symbol(s) => Some(s),
        //     ValueKind::Object(o) => o.internal_slots(vm),
        //     ValueKind::External(_) => unreachable!(),
        // }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ValueKind {
    /// The number type
    Number(Number),
    /// The boolean type
    Boolean(bool),
    /// The string type
    String(JsString),
    /// The undefined type
    Undefined(Undefined),
    /// The null type
    Null(Null),
    /// The symbol type
    Symbol(Symbol),
    /// The object type
    Object(ObjectId),
    /// An "external" value that is being used by other functions.
    External(ExternalValue),
}

/// A wrapper type around JavaScript values that are not rooted.
///
/// Before accessing a value, you must root it using a [`LocalScope`],
/// to prevent a situation in which the garbage collector collects
/// the value while it is still being used.
#[derive(Debug, Copy, Clone)]
pub struct Unrooted {
    // Possible mini optimization: store a flag that indicates if the value is already rooted?
    value: Value,
}

impl Unrooted {
    pub fn new(value: Value) -> Self {
        Self { value }
    }

    /// Returns an unprotected, unrooted reference to the value.
    ///
    /// # Safety
    /// The contained value will be returned without first rooting it, so you must ensure that a GC cycle will not
    /// occur.
    pub unsafe fn get(&self) -> &Value {
        &self.value
    }

    /// "Unwraps" the value, no longer protecting you from a GC sweep killing this value.
    ///
    /// # Safety
    /// The contained value will be returned without first rooting it, so you must ensure that a GC cycle will not
    /// occur.
    pub unsafe fn into_value(self) -> Value {
        self.value
    }

    /// Returns the inner value if it is a primitive that does not have anything that must be preserved by the GC
    pub fn try_prim(self) -> Option<Value> {
        if let ValueKind::Boolean(_) | ValueKind::Null(_) | ValueKind::Number(_) | ValueKind::Undefined(_) =
            self.value.unpack()
        {
            Some(self.value)
        } else {
            None
        }
    }
}

pub trait Root {
    type Rooted;
    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    fn root(self, scope: &mut LocalScope<'_>) -> Self::Rooted;
}

pub mod root_ext {
    use crate::localscope::LocalScope;

    use super::Root;

    pub trait RootOkExt<T: Root, E> {
        fn root_ok(self, scope: &mut LocalScope<'_>) -> Result<T::Rooted, E>;
    }

    impl<T: Root, E> RootOkExt<T, E> for Result<T, E> {
        fn root_ok(self, scope: &mut LocalScope<'_>) -> Result<T::Rooted, E> {
            match self {
                Ok(v) => Ok(v.root(scope)),
                Err(e) => Err(e),
            }
        }
    }

    pub trait RootErrExt<T, E: Root> {
        fn root_err(self, scope: &mut LocalScope<'_>) -> Result<T, E::Rooted>;
    }

    impl<T, E: Root> RootErrExt<T, E> for Result<T, E> {
        fn root_err(self, scope: &mut LocalScope<'_>) -> Result<T, E::Rooted> {
            match self {
                Ok(v) => Ok(v),
                Err(e) => Err(e.root(scope)),
            }
        }
    }
}

impl Root for Unrooted {
    type Rooted = Value;
    fn root(self, scope: &mut LocalScope<'_>) -> Self::Rooted {
        scope.add_value(self.value.clone());
        self.value
    }
}

impl<T: Root, E: Root> Root for Result<T, E> {
    type Rooted = Result<T::Rooted, E::Rooted>;

    fn root(self, scope: &mut LocalScope<'_>) -> Self::Rooted {
        match self {
            Ok(v) => Ok(v.root(scope)),
            Err(e) => Err(e.root(scope)),
        }
    }
}

impl<T: Root> Root for Option<T> {
    type Rooted = Option<T::Rooted>;

    fn root(self, scope: &mut LocalScope<'_>) -> Self::Rooted {
        self.map(|v| v.root(scope))
    }
}

impl From<Value> for Unrooted {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

/// Usually you do not need to worry about this type in e.g. JS handler functions,
/// since we unbox values on use in the Vm directly.
#[derive(Debug, Trace, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ExternalValue {
    // The `dyn Object` is always `Value` (an invariant of this type).
    // It's currently type-erased, but there's no real reason for this. This should make the transition
    // to a thin `Handle` with its vtable in the allocation easier.
    inner: ObjectId,
}

impl ExternalValue {
    /// The `dyn Object` *must* be `Value`.
    // can we make this type safe?
    pub fn new(vm: &Vm, inner: ObjectId) -> Self {
        // debug_assert!(inner.as_any().downcast_ref::<Value>().is_some());
        Self { inner }
    }

    pub fn inner(&self) -> Value {
        todo!("Object::as_any needs to take a &Vm")
        // self.inner
        //     .as_any()
        //     .downcast_ref()
        //     .expect("invariant violated: ExternalValue did not contain a Value")
    }

    pub fn id(&self) -> ObjectId {
        self.inner
    }

    // pub fn as_gc_handle(&self) -> Handle {
    //     self.inner.clone()
    // }

    /// Assigns a new value to this external.
    ///
    /// # Safety
    /// Callers must ensure that the value being replaced does not have active borrows.
    /// You also must not have any downcasted `Handle` (e.g. `Handle<str>`)
    /// as the type might change with this replace.
    ///
    /// For example, this usage is Undefined Behavior:
    /// ```ignore
    /// let ext = ExternalValue::new(...);
    /// let r: &Value = ext.inner();
    /// ExternalValue::replace(&ext, Value::Number(4.0)); // UB, writing to the inner value while a borrow is live
    /// use_borrow(r);
    /// ```
    pub unsafe fn replace(this: &ExternalValue, value: Value) {
        // Even though it looks like we are assigning through a shared reference,
        // this is ok because Handle has a mutable pointer to the GcNode on the heap

        todo!();
        // assert_eq!(this.inner.as_any().type_id(), TypeId::of::<Value>());
        // // SAFETY: casting to *mut GcNode<Value>, then dereferencing + writing
        // // to said pointer is safe, because it is asserted on the previous line that the type is correct
        // (*this.inner.as_ptr::<Value>()).value.0 = value;
    }
}

impl Object for ExternalValue {
    delegate!(
        inner,
        set_property,
        delete_property,
        set_prototype,
        own_keys,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        get_prototype,
        type_of,
        internal_slots
    );

    // NB: this intentionally does not delegate to self.inner.as_any() because
    // we need to downcast to ExternalValue specifically in some places.
    // for that reason, prefer calling downcast_ref not on handles directly
    // but on values.
    fn as_any(&self, _: &Vm) -> &dyn std::any::Any {
        self
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.inner.apply(scope, this, args)
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        _callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.inner.construct(scope, this, args)
    }
}

impl Value {
    pub fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.get_property(sc, key),
            ValueKind::Number(n) => n.get_property(sc, self.clone(), key),
            ValueKind::Boolean(b) => b.get_property(sc, self.clone(), key),
            ValueKind::String(s) => s.get_property(sc, self.clone(), key),
            ValueKind::External(o) => o.inner().get_property(sc, key),
            ValueKind::Undefined(u) => u.get_property(sc, self.clone(), key),
            ValueKind::Null(n) => n.get_property(sc, self.clone(), key),
            ValueKind::Symbol(s) => s.get_property(sc, self.clone(), key),
        }
    }

    pub fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.apply(sc, this, args),
            ValueKind::External(o) => o.inner().apply(sc, this, args),
            ValueKind::Number(n) => throw!(sc, TypeError, "{} is not a function", n),
            ValueKind::Boolean(b) => throw!(sc, TypeError, "{} is not a function", b),
            ValueKind::String(s) => {
                let s = s.res(sc).to_owned();
                throw!(sc, TypeError, "{} is not a function", s)
            }
            ValueKind::Undefined(_) => throw!(sc, TypeError, "undefined is not a function"),
            ValueKind::Null(_) => throw!(sc, TypeError, "null is not a function"),
            ValueKind::Symbol(s) => throw!(sc, TypeError, "{:?} is not a function", s),
        }
    }

    /// Calls a function with debug information. This will print the function being attempted to call as written in the source code.
    pub(crate) fn apply_with_debug(
        &self,
        sc: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
        ip: u16,
    ) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.apply(sc, this, args),
            ValueKind::External(o) => o.inner().apply(sc, this, args),
            _ => {
                cold_path();

                let frame = sc.active_frame();
                let snippet = frame
                    .function
                    .debug_symbols
                    .get(ip)
                    .res(&frame.function.source)
                    .to_owned();

                throw!(sc, TypeError, "{} is not a function", snippet)
            }
        }
    }

    pub fn construct(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.construct(sc, this, args),
            ValueKind::External(o) => o.inner().construct(sc, this, args),
            ValueKind::Number(n) => throw!(sc, TypeError, "{} is not a constructor", n),
            ValueKind::Boolean(b) => throw!(sc, TypeError, "{} is not a constructor", b),
            ValueKind::String(s) => {
                let s = s.res(sc).to_owned();
                throw!(sc, TypeError, "{} is not a constructor", s)
            }
            ValueKind::Undefined(_) => throw!(sc, TypeError, "undefined is not a constructor"),
            ValueKind::Null(_) => throw!(sc, TypeError, "null is not a constructor"),
            ValueKind::Symbol(s) => throw!(sc, TypeError, "{:?} is not a constructor", s),
        }
    }

    pub fn is_truthy(&self, sc: &mut LocalScope<'_>) -> bool {
        match self.unpack() {
            ValueKind::Boolean(b) => b,
            ValueKind::String(s) => !s.res(sc).is_empty(),
            ValueKind::Number(Number(n)) => n != 0.0 && !n.is_nan(),
            ValueKind::Symbol(_) => true,
            ValueKind::Object(_) => true,
            ValueKind::Undefined(_) => false,
            ValueKind::Null(_) => false,
            ValueKind::External(_) => panic!("called is_truthy on an external; consider unboxing it first"),
        }
    }

    pub fn is_nullish(&self) -> bool {
        match self.unpack() {
            ValueKind::Null(_) => true,
            ValueKind::Undefined(_) => true,
            ValueKind::External(_) => panic!("called is_nullish on an external; consider unboxing it first"),
            _ => false,
        }
    }

    pub fn unbox_external(self) -> Value {
        match self.unpack() {
            ValueKind::External(e) => e.inner(),
            _ => self,
        }
    }

    pub fn into_option(self) -> Option<Self> {
        match self.unpack() {
            ValueKind::Undefined(_) => None,
            _ => Some(self),
        }
    }

    pub fn for_each_prototype<T>(
        &self,
        sc: &mut LocalScope<'_>,
        mut fun: impl FnMut(&mut LocalScope<'_>, &Value) -> Result<ControlFlow<T>, Value>,
    ) -> Result<ControlFlow<T>, Value> {
        let mut this = self.clone();
        while !matches!(this.unpack(), ValueKind::Null(_)) {
            if let ControlFlow::Break(b) = fun(sc, &this)? {
                return Ok(ControlFlow::Break(b));
            }
            this = this.get_prototype(sc)?;
        }
        Ok(ControlFlow::Continue(()))
    }

    pub fn instanceof(&self, ctor: &Self, sc: &mut LocalScope) -> Result<bool, Value> {
        if !matches!(self.unpack(), ValueKind::Object(_)) {
            return Ok(false);
        }

        // Look if self[prototype] == ctor.prototype, repeat for all objects in self's prototype chain
        let target_proto = ctor.get_property(sc, sym::prototype.into()).root(sc)?;
        self.for_each_prototype(sc, |_, proto| {
            Ok(if proto == &target_proto {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            })
        })
        .map(|c| c.is_break())
    }

    /// Attempts to downcast this value to a concrete type `T`.
    ///
    /// NOTE: if this value is an external, it will call downcast_ref on the "lower level" handle (i.e. the wrapped object)
    pub fn downcast_ref<T: 'static>(&self, vm: &Vm) -> Option<&T> {
        // weird borrowck error
        todo!()

        // match self.unpack() {
        //     ValueKind::Object(obj) => obj.as_any(vm).downcast_ref(),
        //     ValueKind::External(obj) => obj.inner.as_any(vm).downcast_ref(),
        //     _ => None,
        // }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
            Self::Undefined => Value::string(sym::undefined.into()),
            Self::Object => Value::string(sym::object.into()),
            Self::Boolean => Value::string(sym::boolean.into()),
            Self::Number => Value::string(sym::number.into()),
            Self::Bigint => Value::string(sym::bigint.into()),
            Self::String => Value::string(sym::string.into()),
            Self::Symbol => Value::string(sym::symbol.into()),
            Self::Function => Value::string(sym::function.into()),
        }
    }
}

pub trait ValueContext {
    fn unwrap_or_undefined(self) -> Value;
    fn unwrap_or_null(self) -> Value;
    // fn context<S: Into<Rc<str>>>(self, vm: &mut Vm, message: S) -> Result<Value, Value>;
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

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        sc.impure_builtins();
        self.inner.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        sc.impure_builtins();
        self.inner.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        sc.impure_builtins();
        self.inner.set_prototype(sc, value)
    }

    fn as_any(&self, _: &Vm) -> &dyn std::any::Any {
        &self.inner
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.inner.own_keys(sc)
    }

    fn internal_slots(&self, vm: &Vm) -> Option<&dyn InternalSlots> {
        self.inner.internal_slots(vm)
    }
}
