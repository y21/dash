pub mod arguments;
pub mod array;
pub mod arraybuffer;
pub mod boxed;
pub mod conversions;
pub mod date;
pub mod error;
pub mod function;
pub mod map;
pub mod object;
pub mod ops;
pub mod primitive;
pub mod promise;
pub mod regex;
pub mod set;
pub mod typedarray;

use std::any::TypeId;
use std::ops::ControlFlow;
use std::ptr;

use dash_middle::interner::{self, sym};
use dash_middle::util::ThreadSafeStorage;
use dash_proc_macro::Trace;

pub mod string;
use crate::frame::This;
use crate::gc::ObjectId;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::util::cold_path;
use crate::value::primitive::{Null, Undefined};
use crate::{Vm, delegate, throw};

use self::object::{Object, PropertyKey, PropertyValue};
use self::primitive::{InternalSlots, Number, Symbol};
use self::string::JsString;
use super::localscope::LocalScope;

/// Implements the `Object::extract_type_raw` function by checking the type id of the implementer and the provided fields
#[macro_export]
macro_rules! extract {
    (self $(,$field:ident)*) => {
        fn extract_type_raw(&self, #[allow(unused_variables)] vm: &$crate::Vm, type_id: std::any::TypeId) -> Option<std::ptr::NonNull<()>> {
            if std::any::TypeId::of::<Self>() == type_id {
                Some(std::ptr::NonNull::from(self).cast())
            }
            $(
                else if let Some(v) = $crate::value::object::Object::extract_type_raw(&self.$field, vm, type_id) {
                    Some(v)
                }
            )*
            else {
                None
            }
        }
    };
    ($($field:ident),*) => {
        fn extract_type_raw(&self, vm: &$crate::Vm, type_id: std::any::TypeId) -> Option<std::ptr::NonNull<()>> {
            $(
                if let Some(v) = $crate::value::object::Object::extract_type_raw(&self.$field, vm, type_id) {
                    return Some(v);
                }
            )*
            None
        }
    };
}

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
    /*
    Packed value encoding scheme (high 13 bits):
    0b0_11111111111_10 00	NaN
    0b1_11111111111_00 00	-inf
    0b0_11111111111_00 00	inf
    0bx_11111111111_x0 00	<normal double>
    0bx_xxxxxxxxxxx_xx xx	<normal double> (at least one of the exponent bits is 0)

    0b0_11111111111_11 00	boolean
    0b0_11111111111_11 10	string
    0b1_11111111111_11 10	symbol
    0b0_11111111111_11 01	undefined
    0b1_11111111111_11 01	null
    0b0_11111111111_11 10	object
    0b1_11111111111_11 10	external
                     ^ This bit is always set for non-doubles
     */
    const TAG_MASK: u64 = 0b1_11111111111_1111 << (64 - 16);
    const NON_NUMBER_MASK: u64 = 0b0_11111111111_1100 << (64 - 16);

    pub(crate) const BOOLEAN_MASK: u64 = 0b0_11111111111_1100 << (64 - 16);
    pub(crate) const STRING_MASK: u64 = 0b1_11111111111_1100 << (64 - 16);
    pub(crate) const SYMBOL_MASK: u64 = 0b0_11111111111_1101 << (64 - 16);
    pub(crate) const UNDEFINED_MASK: u64 = 0b1_11111111111_1101 << (64 - 16);
    pub(crate) const NULL_MASK: u64 = 0b0_11111111111_1110 << (64 - 16);
    pub(crate) const OBJECT_MASK: u64 = 0b1_11111111111_1110 << (64 - 16);
    pub(crate) const EXTERNAL_MASK: u64 = 0b0_11111111111_1111 << (64 - 16);

    pub fn number(v: f64) -> Self {
        #[cold]
        #[inline(never)]
        fn fail(bits: u64, val: f64) -> ! {
            panic!("invalid float bitpattern: {bits} ({val})")
        }

        let bits = v.to_bits();
        if (bits & Self::NON_NUMBER_MASK) == Self::NON_NUMBER_MASK {
            fail(bits, v);
        }
        Self(bits)
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
            _ if (self.0 & Self::NON_NUMBER_MASK) != Self::NON_NUMBER_MASK => {
                ValueKind::Number(Number(f64::from_bits(self.0)))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
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

// TODO: can we just get rid of this impl if ExternalValue stores ValueId instead of ObjectId which should remove the need for having an object vtable for value
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

    fn apply(&self, scope: &mut LocalScope, _: ObjectId, this: This, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        self.apply(scope, this, args)
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
        this: This,
        args: Vec<Value>,
        new_target: ObjectId,
    ) -> Result<Unrooted, Unrooted> {
        self.construct_with_target(scope, this, args, new_target)
    }

    fn internal_slots(&self, _: &Vm) -> Option<&dyn InternalSlots> {
        Some(self)
    }

    fn extract_type_raw(&self, vm: &Vm, type_id: TypeId) -> Option<ptr::NonNull<()>> {
        if TypeId::of::<Self>() == type_id {
            Some(ptr::NonNull::from(self).cast())
        } else {
            match self.unpack() {
                ValueKind::Number(number) => number.extract_type_raw(vm, type_id),
                ValueKind::Boolean(boolean) => boolean.extract_type_raw(vm, type_id),
                ValueKind::String(js_string) => js_string.extract_type_raw(vm, type_id),
                ValueKind::Undefined(undefined) => undefined.extract_type_raw(vm, type_id),
                ValueKind::Null(null) => null.extract_type_raw(vm, type_id),
                ValueKind::Symbol(symbol) => symbol.extract_type_raw(vm, type_id),
                ValueKind::Object(alloc_id) => alloc_id.extract_type_raw(vm, type_id),
                ValueKind::External(external_value) => external_value.extract_type_raw(vm, type_id),
            }
        }
    }
}

#[deny(clippy::missing_trait_methods)]
impl InternalSlots for Value {
    fn string_value(&self, vm: &Vm) -> Option<JsString> {
        match self.unpack() {
            ValueKind::String(s) => Some(s),
            ValueKind::Object(obj) => obj.internal_slots(vm).and_then(|slots| slots.string_value(vm)),
            ValueKind::External(ext) => ext.internal_slots(vm).and_then(|slots| slots.string_value(vm)),
            _ => None,
        }
    }

    fn number_value(&self, vm: &Vm) -> Option<f64> {
        match self.unpack() {
            ValueKind::Number(Number(n)) => Some(n),
            ValueKind::Object(obj) => obj.internal_slots(vm).and_then(|slots| slots.number_value(vm)),
            ValueKind::External(ext) => ext.internal_slots(vm).and_then(|slots| slots.number_value(vm)),
            _ => None,
        }
    }

    fn boolean_value(&self, vm: &Vm) -> Option<bool> {
        match self.unpack() {
            ValueKind::Boolean(b) => Some(b),
            ValueKind::Object(obj) => obj.internal_slots(vm).and_then(|slots| slots.boolean_value(vm)),
            ValueKind::External(ext) => ext.internal_slots(vm).and_then(|slots| slots.boolean_value(vm)),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

impl ValueKind {
    /// Attempts to downcast this value to a concrete type `T`.
    ///
    /// NOTE: if this value is an external, it will call downcast_ref on the "lower level" handle (i.e. the wrapped object)
    pub fn downcast_ref<T: 'static>(&self, vm: &Vm) -> Option<&T> {
        // TODO: remove this in favor of extract()
        match self {
            ValueKind::Object(obj) => obj.extract(vm),
            ValueKind::External(obj) => obj.inner.extract(vm),
            _ => None,
        }
    }
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
        scope.add_value(self.value);
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
        // TODO: make a ValueId
        // TODO: how does this interact with values that extend externalvalues? does that even work?
        debug_assert!(inner.extract::<Value>(vm).is_some());
        Self { inner }
    }

    pub fn inner(&self, vm: &Vm) -> Value {
        *self
            .inner
            .extract(vm)
            .expect("invariant violated: ExternalValue did not contain a Value")
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
    pub unsafe fn replace(vm: &mut Vm, this: ExternalValue, value: Value) {
        assert!(this.inner.extract::<Value>(vm).is_some());
        let data = vm.alloc.data(this.inner).cast_mut().cast::<Value>();
        ptr::write(data, value);
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

    extract!(self, inner);

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: ObjectId,
        this: This,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.inner.apply(scope, this, args)
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        _callee: ObjectId,
        this: This,
        args: Vec<Value>,
        new_target: ObjectId,
    ) -> Result<Unrooted, Unrooted> {
        self.inner.construct_with_target(scope, this, args, new_target)
    }
}

impl Value {
    pub fn extract<T: 'static>(&self, vm: &Vm) -> Option<&T> {
        let ptr = self.extract_type_raw(vm, TypeId::of::<T>())?;
        // SAFETY: `extract_type_raw` only returns `Some(_)` for types with the same TypeId
        Some(unsafe { ptr.cast::<T>().as_ref() })
    }

    pub fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.get_property(sc, key),
            // TODO: autobox primitives
            ValueKind::Number(n) => n.get_property(sc, This::Bound(*self), key),
            ValueKind::Boolean(b) => b.get_property(sc, This::Bound(*self), key),
            ValueKind::String(s) => s.get_property(sc, This::Bound(*self), key),
            ValueKind::External(o) => o.inner(sc).get_property(sc, key),
            ValueKind::Undefined(u) => u.get_property(sc, This::Bound(*self), key),
            ValueKind::Null(n) => n.get_property(sc, This::Bound(*self), key),
            ValueKind::Symbol(s) => s.get_property(sc, This::Bound(*self), key),
        }
    }

    pub fn apply(&self, sc: &mut LocalScope, this: This, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.apply(sc, this, args),
            ValueKind::External(o) => o.inner(sc).apply(sc, this, args),
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
        this: This,
        args: Vec<Value>,
        ip: u16,
    ) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.apply(sc, this, args),
            ValueKind::External(o) => o.inner(sc).apply(sc, this, args),
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

    pub fn construct(&self, sc: &mut LocalScope, this: This, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.construct(sc, this, args),
            ValueKind::External(o) => o.inner(sc).construct(sc, this, args),
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

    pub fn construct_with_target(
        &self,
        sc: &mut LocalScope,
        this: This,
        args: Vec<Value>,
        new_target: ObjectId,
    ) -> Result<Unrooted, Unrooted> {
        match self.unpack() {
            ValueKind::Object(o) => o.construct_with_target(sc, this, args, new_target),
            ValueKind::External(o) => o.inner(sc).construct_with_target(sc, this, args, new_target),
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

    pub fn unbox_external(self, vm: &Vm) -> Value {
        match self.unpack() {
            ValueKind::External(e) => e.inner(vm),
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
        let mut this = *self;
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
        self.copied().unwrap_or_undefined()
    }

    fn unwrap_or_null(self) -> Value {
        self.copied().unwrap_or_null()
    }
}

pub trait ExceptionContext<T> {
    fn or_type_err(self, sc: &mut LocalScope<'_>, message: &str) -> Result<T, Value>;
    fn or_err(self, sc: &mut LocalScope<'_>, message: &str) -> Result<T, Value>;
    fn or_type_err_args(self, sc: &mut LocalScope<'_>, args: std::fmt::Arguments) -> Result<T, Value>;
}

impl<T> ExceptionContext<T> for Option<T> {
    fn or_type_err(self, sc: &mut LocalScope<'_>, message: &str) -> Result<T, Value> {
        match self {
            Some(v) => Ok(v),
            None => throw!(sc, TypeError, "{}", message),
        }
    }

    fn or_type_err_args(self, sc: &mut LocalScope<'_>, args: std::fmt::Arguments) -> Result<T, Value> {
        match self {
            Some(v) => Ok(v),
            None => throw!(sc, TypeError, "{:?}", args),
        }
    }

    fn or_err(self, sc: &mut LocalScope<'_>, message: &str) -> Result<T, Value> {
        match self {
            Some(v) => Ok(v),
            None => throw!(sc, TypeError, "{}", message),
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

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.inner.own_keys(sc)
    }

    fn internal_slots(&self, vm: &Vm) -> Option<&dyn InternalSlots> {
        self.inner.internal_slots(vm)
    }

    extract!(inner);
}
