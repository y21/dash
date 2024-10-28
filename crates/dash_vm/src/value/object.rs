use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::BuildHasherDefault;

use crate::gc::persistent::Persistent;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::gc::{ObjectId, ObjectVTable};
use bitflags::bitflags;
use dash_middle::interner::sym;
use dash_proc_macro::Trace;
use hashbrown::hash_map::Entry;
use rustc_hash::FxHasher;

use crate::localscope::LocalScope;
use crate::{throw, Vm};

use super::ops::conversions::ValueConversion;
use super::primitive::{InternalSlots, Symbol};
use super::string::JsString;
use super::{Root, Typeof, Unpack, Unrooted, Value, ValueContext, ValueKind};

pub type ObjectMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;

// only here for the time being, will be removed later
fn __assert_trait_object_safety(_: Box<dyn Object>) {}

pub trait Object: Debug + Trace {
    fn get_own_property(&self, sc: &mut LocalScope, this: Value, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        delegate_get_own_property(self, this, sc, key)
    }

    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted>;

    fn get_property(&self, sc: &mut LocalScope, this: Value, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        delegate_get_property(self, this, sc, key)
    }

    fn get_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        let own_descriptor = self.get_own_property_descriptor(sc, key.clone())?;
        if own_descriptor.is_some() {
            return Ok(own_descriptor);
        }

        match self.get_prototype(sc)?.unpack() {
            ValueKind::Object(object) => object.get_property_descriptor(sc, key),
            ValueKind::External(object) => object.get_own_property_descriptor(sc, key),
            ValueKind::Null(..) => Ok(None),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value>;

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value>;

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value>;

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value>;

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted>;

    fn construct(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.apply(scope, callee, this, args)
    }

    fn as_any(&self, vm: &Vm) -> &dyn Any;

    fn internal_slots(&self, _: &Vm) -> Option<&dyn InternalSlots> {
        None
    }

    // TODO: change this to Vec<JsString>
    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value>;

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Object
    }
}

#[macro_export]
macro_rules! delegate {
    (override $field:ident, get_own_property_descriptor) => {
        fn get_own_property_descriptor(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            key: $crate::value::object::PropertyKey,
        ) -> Result<Option<$crate::value::object::PropertyValue>, $crate::value::Unrooted> {
            self.$field.get_own_property_descriptor(sc, key)
        }
    };
    (override $field:ident, get_property) => {
        fn get_property(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            this: $crate::value::Value,
            key: $crate::value::object::PropertyKey,
        ) -> Result<$crate::value::Unrooted, $crate::value::Unrooted> {
            $crate::value::object::Object::get_property(&self.$field, sc, this, key)
        }
    };
    (override $field:ident, get_property_descriptor) => {
        fn get_property_descriptor(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            key: $crate::value::object::PropertyKey,
        ) -> Result<Option<$crate::value::object::PropertyValue>, $crate::value::Unrooted> {
            self.$field.get_property_descriptor(sc, key)
        }
    };
    (override $field:ident, set_property) => {
        fn set_property(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            key: $crate::value::object::PropertyKey,
            value: $crate::value::object::PropertyValue,
        ) -> Result<(), $crate::value::Value> {
            self.$field.set_property(sc, key, value)
        }
    };
    (override $field:ident, delete_property) => {
        fn delete_property(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            key: $crate::value::object::PropertyKey,
        ) -> Result<$crate::value::Unrooted, $crate::value::Value> {
            self.$field.delete_property(sc, key)
        }
    };
    (override $field:ident, set_prototype) => {
        fn set_prototype(&self, sc: &mut $crate::localscope::LocalScope, value: $crate::value::Value) -> Result<(), $crate::value::Value> {
            self.$field.set_prototype(sc, value)
        }
    };
    (override $field:ident, get_prototype) => {
        fn get_prototype(&self, sc: &mut $crate::localscope::LocalScope) -> Result<$crate::value::Value, $crate::value::Value> {
            self.$field.get_prototype(sc)
        }
    };
    (override $field:ident, as_any) => {
        fn as_any(&self, _: &$crate::Vm) -> &dyn std::any::Any {
            self
        }
    };
    (override $field:ident, own_keys) => {
        fn own_keys(&self, sc: &mut $crate::localscope::LocalScope<'_>) -> Result<Vec<$crate::value::Value>, $crate::value::Value> {
            self.$field.own_keys(sc)
        }
    };
    (override $field:ident, apply) => {
        fn apply(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            id: $crate::gc::ObjectId,
            this: $crate::value::Value,
            args: Vec<$crate::value::Value>,
        ) -> Result<$crate::value::Unrooted, $crate::value::Unrooted> {
            $crate::value::object::Object::apply(&self.$field, sc, id, this, args)
        }
    };
    (override $field:ident, construct) => {
        fn construct(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            id: $crate::gc::ObjectId,
            this: $crate::value::Value,
            args: Vec<$crate::value::Value>,
        ) -> Result<$crate::value::Unrooted, $crate::value::Unrooted> {
            $crate::value::object::Object::construct(&self.$field, sc, id, this, args)
        }
    };
    (override $field:ident, type_of) => {
        fn type_of(&self, vm: &Vm) -> $crate::value::Typeof {
            self.$field.type_of(vm)
        }
    };
    (override $field:ident, internal_slots) => {
        fn internal_slots(&self, vm: &Vm) -> Option<&dyn InternalSlots> {
            self.$field.internal_slots(vm)
        }
    };

    ($field:ident, $($method:ident),* $(,)?) => {
        $(
            $crate::delegate!(override $field, $method);
        )*
    };
}

#[derive(Debug, Clone)]
pub struct NamedObject {
    prototype: RefCell<Option<ObjectId>>,
    constructor: RefCell<Option<ObjectId>>,
    values: RefCell<ObjectMap<PropertyKey, PropertyValue>>,
}

// TODO: optimization opportunity: some kind of Number variant for faster indexing without .to_string()
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PropertyKey {
    String(JsString),
    Symbol(Symbol),
}

unsafe impl Trace for PropertyKey {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            PropertyKey::String(s) => s.trace(cx),
            PropertyKey::Symbol(s) => s.trace(cx),
        }
    }
}

bitflags! {
    pub struct PropertyDataDescriptor: u8 {
        const CONFIGURABLE = 1 << 0;
        const ENUMERABLE = 1 << 1;
        const WRITABLE = 1 << 2;
    }
}

unsafe impl Trace for PropertyDataDescriptor {
    fn trace(&self, _: &mut TraceCtxt<'_>) {}
}

impl Default for PropertyDataDescriptor {
    fn default() -> Self {
        Self::CONFIGURABLE | Self::ENUMERABLE | Self::WRITABLE
    }
}

#[derive(Debug, Clone, Trace, PartialEq, Eq)]
pub struct PropertyValue {
    pub kind: PropertyValueKind,
    pub descriptor: PropertyDataDescriptor,
}

impl PropertyValue {
    pub fn new(kind: PropertyValueKind, descriptor: PropertyDataDescriptor) -> Self {
        Self { kind, descriptor }
    }

    /// Convenience function for creating a static property with a default descriptor (all bits set to 1)
    pub fn static_default(value: Value) -> Self {
        Self::new(PropertyValueKind::Static(value), Default::default())
    }

    /// Convenience function for creating a static property with an empty descriptor (all bits set to 0)
    pub fn static_empty(value: Value) -> Self {
        Self::new(PropertyValueKind::Static(value), PropertyDataDescriptor::empty())
    }

    /// Convenience function for creating a static, non-enumerable property
    pub fn static_non_enumerable(value: Value) -> Self {
        Self::new(
            PropertyValueKind::Static(value),
            PropertyDataDescriptor::WRITABLE | PropertyDataDescriptor::CONFIGURABLE,
        )
    }

    pub fn getter_default(value: ObjectId) -> Self {
        Self::new(
            PropertyValueKind::Trap {
                get: Some(value),
                set: None,
            },
            Default::default(),
        )
    }

    pub fn setter_default(value: ObjectId) -> Self {
        Self::new(
            PropertyValueKind::Trap {
                get: None,
                set: Some(value),
            },
            Default::default(),
        )
    }

    pub fn kind(&self) -> &PropertyValueKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut PropertyValueKind {
        &mut self.kind
    }

    pub fn into_parts(self) -> (PropertyValueKind, PropertyDataDescriptor) {
        (self.kind, self.descriptor)
    }

    pub fn into_kind(self) -> PropertyValueKind {
        self.kind
    }

    pub fn get_or_apply(&self, sc: &mut LocalScope, this: Value) -> Result<Unrooted, Unrooted> {
        self.kind.get_or_apply(sc, this)
    }

    pub fn to_descriptor_value(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        let obj = NamedObject::new(sc);

        match &self.kind {
            PropertyValueKind::Static(value) => {
                obj.set_property(sc, sym::value.into(), PropertyValue::static_default(value.clone()))?;
            }
            PropertyValueKind::Trap { get, set } => {
                let get = get.map(Value::object).unwrap_or_undefined();
                let set = set.map(Value::object).unwrap_or_undefined();
                obj.set_property(sc, sym::get.into(), PropertyValue::static_default(get))?;
                obj.set_property(sc, sym::set.into(), PropertyValue::static_default(set))?;
            }
        }

        obj.set_property(
            sc,
            sym::writable.into(),
            PropertyValue::static_default(Value::boolean(
                self.descriptor.contains(PropertyDataDescriptor::WRITABLE),
            )),
        )?;

        obj.set_property(
            sc,
            sym::enumerable.into(),
            PropertyValue::static_default(Value::boolean(
                self.descriptor.contains(PropertyDataDescriptor::ENUMERABLE),
            )),
        )?;

        obj.set_property(
            sc,
            sym::configurable.into(),
            PropertyValue::static_default(Value::boolean(
                self.descriptor.contains(PropertyDataDescriptor::CONFIGURABLE),
            )),
        )?;

        Ok(Value::object(sc.register(obj)))
    }

    pub fn from_descriptor_value(sc: &mut LocalScope<'_>, value: Value) -> Result<Self, Value> {
        let mut flags = PropertyDataDescriptor::empty();
        let configurable = value.get_property(sc, sym::configurable.into()).root(sc)?.into_option();
        let enumerable = value.get_property(sc, sym::enumerable.into()).root(sc)?.into_option();
        let writable = value.get_property(sc, sym::writable.into()).root(sc)?.into_option();

        if configurable.is_some_and(|v| v.is_truthy(sc)) {
            flags |= PropertyDataDescriptor::CONFIGURABLE;
        }
        if enumerable.is_some_and(|v| v.is_truthy(sc)) {
            flags |= PropertyDataDescriptor::ENUMERABLE;
        }
        if writable.is_some_and(|v| v.is_truthy(sc)) {
            flags |= PropertyDataDescriptor::WRITABLE;
        }

        // TODO: make sure that if value is set, get/set are not

        let static_value = value.get_property(sc, sym::value.into()).root(sc)?.into_option();
        let kind = match static_value {
            Some(static_value) => PropertyValueKind::Static(static_value),
            None => {
                let get = value
                    .get_property(sc, sym::get.into())
                    .root(sc)?
                    .into_option()
                    .and_then(|v| match v.unpack() {
                        ValueKind::Object(o) => Some(o),
                        _ => None,
                    });
                let set = value
                    .get_property(sc, sym::set.into())
                    .root(sc)?
                    .into_option()
                    .and_then(|v| match v.unpack() {
                        ValueKind::Object(o) => Some(o),
                        _ => None,
                    });

                PropertyValueKind::Trap { get, set }
            }
        };

        Ok(Self::new(kind, flags))
    }
}

// TODO: these handles should be "hidden" behind something similar to the `Unrooted` value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValueKind {
    /// Accessor property
    Trap {
        get: Option<ObjectId>,
        set: Option<ObjectId>,
    },
    /// Static value property
    Static(Value),
    // TODO: magic property that appears "static" but is actually computed, e.g. array.length
}

impl PropertyValueKind {
    pub fn getter(get: ObjectId) -> Self {
        Self::Trap {
            get: Some(get),
            set: None,
        }
    }

    pub fn setter(set: ObjectId) -> Self {
        Self::Trap {
            set: Some(set),
            get: None,
        }
    }

    pub fn as_static(&self) -> Option<&Value> {
        match self {
            Self::Static(val) => Some(val),
            _ => None,
        }
    }

    pub fn get_or_apply(&self, sc: &mut LocalScope, this: Value) -> Result<Unrooted, Unrooted> {
        match self {
            Self::Static(value) => Ok(value.clone().into()),
            Self::Trap { get, .. } => match get {
                Some(id) => id.apply(sc, this, Vec::new()),
                None => Ok(Value::undefined().into()),
            },
        }
    }
}

unsafe impl Trace for PropertyValueKind {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            Self::Static(value) => value.trace(cx),
            Self::Trap { get, set } => {
                if let Some(get) = get {
                    get.trace(cx);
                }
                if let Some(set) = set {
                    set.trace(cx);
                }
            }
        }
    }
}

impl PropertyKey {
    pub fn as_string(&self) -> Option<JsString> {
        match self {
            PropertyKey::String(s) => Some(*s),
            _ => None,
        }
    }
}

impl From<JsString> for PropertyKey {
    fn from(value: JsString) -> Self {
        PropertyKey::String(value)
    }
}
impl From<dash_middle::interner::Symbol> for PropertyKey {
    fn from(value: dash_middle::interner::Symbol) -> Self {
        PropertyKey::String(value.into())
    }
}

impl From<Symbol> for PropertyKey {
    fn from(s: Symbol) -> Self {
        PropertyKey::Symbol(s)
    }
}

impl PropertyKey {
    pub fn as_value(&self) -> Value {
        match self {
            PropertyKey::String(s) => Value::string(*s),
            PropertyKey::Symbol(s) => Value::symbol(s.clone()),
        }
    }

    pub fn from_value(sc: &mut LocalScope, value: Value) -> Result<Self, Value> {
        // TODO: call ToPrimitive as specified by ToPropertyKey in the spec?
        match value.unpack() {
            ValueKind::Symbol(s) => Ok(Self::Symbol(s)),
            _ => Ok(PropertyKey::String(value.to_js_string(sc)?)),
        }
    }
}

impl NamedObject {
    pub fn new(vm: &Vm) -> Self {
        Self::with_values(vm, ObjectMap::default())
    }

    pub fn with_values(vm: &Vm, values: ObjectMap<PropertyKey, PropertyValue>) -> Self {
        let objp = vm.statics.object_prototype.clone();
        let objc = vm.statics.object_ctor.clone(); // TODO: function_ctor instead

        Self {
            prototype: RefCell::new(Some(objp)),
            constructor: RefCell::new(Some(objc)),
            values: RefCell::new(values),
        }
    }

    /// Creates an empty object with a null prototype
    pub fn null() -> Self {
        Self {
            prototype: RefCell::new(None),
            constructor: RefCell::new(None),
            values: RefCell::new(ObjectMap::default()),
        }
    }

    pub fn null_with_values(values: ObjectMap<PropertyKey, PropertyValue>) -> Self {
        Self {
            prototype: RefCell::new(None),
            constructor: RefCell::new(None),
            values: RefCell::new(values),
        }
    }

    pub fn with_prototype_and_constructor(prototype: ObjectId, ctor: ObjectId) -> Self {
        Self {
            constructor: RefCell::new(Some(ctor)),
            prototype: RefCell::new(Some(prototype)),
            values: RefCell::new(ObjectMap::default()),
        }
    }

    pub fn get_raw_property(&self, pk: PropertyKey) -> Option<PropertyValue> {
        self.values.borrow().get(&pk).cloned()
    }
}

unsafe impl Trace for NamedObject {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        let Self {
            prototype,
            constructor,
            values,
        } = self;
        values.trace(cx);
        prototype.trace(cx);
        constructor.trace(cx);
    }
}

impl Object for NamedObject {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        if let PropertyKey::String(st) = &key {
            match st.sym() {
                sym::__proto__ => return Ok(Some(PropertyValue::static_default(self.get_prototype(sc)?))),
                sym::constructor => {
                    return Ok(Some(PropertyValue::static_default(
                        self.constructor.borrow().map(Value::object).unwrap_or_undefined(),
                    )));
                }
                _ => {}
            }
        };

        let values = self.values.borrow();
        if let Some(value) = values.get(&key).cloned() {
            return Ok(Some(value));
        }

        Ok(None)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        match key.as_string().map(JsString::sym) {
            Some(sym::__proto__) => {
                return self.set_prototype(
                    sc,
                    match value.into_kind() {
                        PropertyValueKind::Static(value) => value,
                        _ => throw!(sc, TypeError, "Prototype cannot be a trap"),
                    },
                );
            }
            Some(sym::constructor) => {
                let obj = if let PropertyValueKind::Static(val) = value.kind {
                    match val.unpack() {
                        ValueKind::Object(obj) => Some(obj),
                        ValueKind::External(obj) => Some(obj.inner),
                        _ => None,
                    }
                } else {
                    None
                };
                let Some(obj) = obj else {
                    throw!(sc, TypeError, "constructor is not an object") // TODO: it doesn't need to be
                };

                self.constructor.replace(Some(obj));
                return Ok(());
            }
            _ => {}
        };

        // TODO: check if we are invoking a setter

        let mut map = self.values.borrow_mut();
        match map.entry(key) {
            Entry::Occupied(mut entry) => {
                if entry.get().descriptor.contains(PropertyDataDescriptor::WRITABLE) {
                    entry.insert(value);
                }
            }
            Entry::Vacant(vacant) => drop(vacant.insert(value)),
        }
        Ok(())
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        let mut values = self.values.borrow_mut();
        let value = values.remove(&key);

        match value.map(PropertyValue::into_kind) {
            Some(PropertyValueKind::Static(value)) => {
                // If a GC'd value is being removed, put it in the LocalScope so it doesn't get removed too early
                // Actually, no need, now that we have `Unrooted`, which requires re-rooting at call site
                // sc.add_value(value.clone());
                Ok(Unrooted::new(value))
            }
            Some(PropertyValueKind::Trap { get, set }) => {
                // Accessors need to be added to the LocalScope too
                if let Some(v) = get {
                    sc.add_ref(v)
                }
                if let Some(v) = set {
                    sc.add_ref(v)
                }

                // Kind of unclear what to return here...
                // We can't invoke the getters/setters
                Ok(Unrooted::new(Value::undefined()))
            }
            None => Ok(Unrooted::new(Value::undefined())),
        }
    }

    fn apply(
        &self,
        _sc: &mut LocalScope,
        _handle: ObjectId,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        Ok(Value::undefined().into())
    }

    fn as_any(&self, _: &Vm) -> &dyn Any {
        self
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        match value.unpack() {
            ValueKind::Null(_) => self.prototype.replace(None),
            ValueKind::Object(handle) => self.prototype.replace(Some(handle)),
            ValueKind::External(handle) => self.prototype.replace(Some(handle.inner)), // TODO: check that handle is an object
            _ => throw!(sc, TypeError, "prototype must be an object"),
        };

        Ok(())
    }

    fn get_prototype(&self, _sc: &mut LocalScope) -> Result<Value, Value> {
        let prototype = self.prototype.borrow();
        match *prototype {
            Some(id) => Ok(Value::object(id)),
            None => Ok(Value::null()),
        }
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        let values = self.values.borrow();
        Ok(values.keys().map(PropertyKey::as_value).collect())
    }
}

// TODO: is this still needed?
impl Object for Box<dyn Object> {
    fn get_own_property(&self, sc: &mut LocalScope, this: Value, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        (**self).get_own_property(sc, this, key)
    }

    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        (**self).get_own_property_descriptor(sc, key)
    }

    fn get_property(&self, sc: &mut LocalScope, this: Value, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        (**self).get_property(sc, this, key)
    }

    fn get_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        (**self).get_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        (**self).set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        (**self).delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        (**self).set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        (**self).get_prototype(sc)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        (**self).apply(scope, callee, this, args)
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        (**self).construct(scope, callee, this, args)
    }

    fn as_any(&self, _: &Vm) -> &dyn Any {
        self
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        (**self).own_keys(sc)
    }

    fn type_of(&self, vm: &Vm) -> Typeof {
        (**self).type_of(vm)
    }

    fn internal_slots(&self, vm: &Vm) -> Option<&dyn InternalSlots> {
        (**self).internal_slots(vm)
    }
}

impl ObjectId {
    pub fn vtable(self, vm: &Vm) -> &'static ObjectVTable {
        unsafe { *vm.alloc.metadata(self) }
    }
    pub fn data_ptr(self, vm: &Vm) -> *const () {
        vm.alloc.data(self)
    }
}

// TODO: can these be inherent methods? or do we actually require the `ObjectId: Object` trait obligation anywhere?
// then they also wouldn't need to take &self
impl Object for ObjectId {
    fn get_own_property(&self, sc: &mut LocalScope, this: Value, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        unsafe { (self.vtable(sc).js_get_own_property)(self.data_ptr(sc), sc, this, key) }
    }

    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        unsafe { (self.vtable(sc).js_get_own_property_descriptor)(self.data_ptr(sc), sc, key) }
    }

    fn get_property(&self, sc: &mut LocalScope, this: Value, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        unsafe { (self.vtable(sc).js_get_property)(self.data_ptr(sc), sc, this, key) }
    }

    fn get_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        unsafe { (self.vtable(sc).js_get_property_descriptor)(self.data_ptr(sc), sc, key) }
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        unsafe { (self.vtable(sc).js_set_property)(self.data_ptr(sc), sc, key, value) }
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        unsafe { (self.vtable(sc).js_delete_property)(self.data_ptr(sc), sc, key) }
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        unsafe { (self.vtable(sc).js_set_prototype)(self.data_ptr(sc), sc, value) }
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        unsafe { (self.vtable(sc).js_get_prototype)(self.data_ptr(sc), sc) }
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        unsafe { (self.vtable(scope).js_apply)(self.data_ptr(scope), scope, callee, this, args) }
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        unsafe { (self.vtable(scope).js_construct)(self.data_ptr(scope), scope, callee, this, args) }
    }

    fn as_any(&self, vm: &Vm) -> &dyn Any {
        unsafe { &*(self.vtable(vm).js_as_any)(self.data_ptr(vm), vm) }
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        unsafe { (self.vtable(sc).js_own_keys)(self.data_ptr(sc), sc) }
    }

    fn type_of(&self, vm: &Vm) -> Typeof {
        unsafe { (self.vtable(vm).js_type_of)(self.data_ptr(vm), vm) }
    }

    fn internal_slots(&self, vm: &Vm) -> Option<&dyn InternalSlots> {
        unsafe { (self.vtable(vm).js_internal_slots)(self.data_ptr(vm), vm).map(|v| &*v) }
    }
}

impl ObjectId {
    pub fn get_property(self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        Object::get_property(&self, sc, Value::object(self), key)
    }

    pub fn get_own_property(self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        Object::get_own_property(&self, sc, Value::object(self), key)
    }

    pub fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        let callee = self.clone();
        Object::apply(self, sc, callee, this, args)
    }

    pub fn construct(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        let callee = self.clone();
        Object::construct(self, sc, callee, this, args)
    }
}

impl Persistent {
    pub fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        self.id().get_property(sc, key)
    }

    pub fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        self.id().apply(sc, this, args)
    }

    pub fn construct(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        self.id().construct(sc, this, args)
    }

    // FIXME: should override typeof, internal_slots, etc.
}

/// Delegates a get_property call to get_property_descriptor and converts the return value respectively
pub fn delegate_get_property<T: Object + ?Sized>(
    this: &T,
    this_value: Value,
    sc: &mut LocalScope,
    key: PropertyKey,
) -> Result<Unrooted, Unrooted> {
    this.get_property_descriptor(sc, key)
        .map(|x| x.unwrap_or_else(|| PropertyValue::static_default(Value::undefined())))
        .and_then(|x| x.get_or_apply(sc, this_value))
}
/// Delegates a get_property call to get_property_descriptor and converts the return value respectively
pub fn delegate_get_own_property<T: Object + ?Sized>(
    this: &T,
    this_value: Value,
    sc: &mut LocalScope,
    key: PropertyKey,
) -> Result<Unrooted, Unrooted> {
    this.get_own_property_descriptor(sc, key)
        .map(|x| x.unwrap_or_else(|| PropertyValue::static_default(Value::undefined())))
        .and_then(|x| x.get_or_apply(sc, this_value))
}
