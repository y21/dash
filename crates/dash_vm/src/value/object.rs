use std::any::TypeId;
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::BuildHasherDefault;
use std::ptr::NonNull;

use crate::frame::This;
use crate::gc::persistent::Persistent;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::gc::{ObjectId, ObjectVTable};
use bitflags::bitflags;
use dash_middle::interner::sym;
use dash_proc_macro::Trace;
use hashbrown::hash_map::Entry;
use rustc_hash::FxHasher;

use crate::localscope::LocalScope;
use crate::{Vm, extract, throw};

use super::function::args::CallArgs;
use super::primitive::InternalSlots;
use super::propertykey::{PropertyKey, ToPropertyKey};
use super::root_ext::RootErrExt;
use super::{Root, Typeof, Unpack, Unrooted, Value, ValueContext, ValueKind};

pub type ObjectMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;

pub trait Object: Debug + Trace {
    fn get_own_property(&self, this: This, key: PropertyKey, sc: &mut LocalScope<'_>) -> Result<Unrooted, Unrooted> {
        delegate_get_own_property(self, this, sc, key)
    }

    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted>;

    fn get_property(&self, this: This, key: PropertyKey, sc: &mut LocalScope<'_>) -> Result<Unrooted, Unrooted> {
        delegate_get_property(self, this, sc, key)
    }

    fn get_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        let own_descriptor = self.get_own_property_descriptor(key, sc)?;
        if own_descriptor.is_some() {
            return Ok(own_descriptor);
        }

        match self.get_prototype(sc)?.unpack() {
            ValueKind::Object(object) => object.get_property_descriptor(key, sc),
            ValueKind::External(object) => object.get_own_property_descriptor(key, sc),
            ValueKind::Null(..) => Ok(None),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope) -> Result<(), Value>;

    fn delete_property(&self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Value>;

    fn set_prototype(&self, value: Value, sc: &mut LocalScope) -> Result<(), Value>;

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value>;

    fn apply(&self, callee: ObjectId, this: This, args: CallArgs, scope: &mut LocalScope)
    -> Result<Unrooted, Unrooted>;

    fn construct(
        &self,
        callee: ObjectId,
        this: This,
        args: CallArgs,
        _new_target: ObjectId,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        self.apply(callee, this, args, scope)
    }

    // TODO: require returning a special kind of pointer wrapper that needs unsafe to construct
    fn extract_type_raw(&self, _: &Vm, _: TypeId) -> Option<NonNull<()>> {
        None
    }

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
            key: $crate::value::propertykey::PropertyKey,
            sc: &mut $crate::localscope::LocalScope,
        ) -> Result<Option<$crate::value::object::PropertyValue>, $crate::value::Unrooted> {
            self.$field.get_own_property_descriptor(key, sc)
        }
    };
    (override $field:ident, get_property) => {
        fn get_property(
            &self,
            this: $crate::frame::This,
            key: $crate::value::propertykey::PropertyKey,
            sc: &mut $crate::localscope::LocalScope,
        ) -> Result<$crate::value::Unrooted, $crate::value::Unrooted> {
            $crate::value::object::Object::get_property(&self.$field, this, key, sc)
        }
    };
    (override $field:ident, get_property_descriptor) => {
        fn get_property_descriptor(
            &self,
            key: $crate::value::propertykey::PropertyKey,
            sc: &mut $crate::localscope::LocalScope,
        ) -> Result<Option<$crate::value::object::PropertyValue>, $crate::value::Unrooted> {
            self.$field.get_property_descriptor(key, sc)
        }
    };
    (override $field:ident, set_property) => {
        fn set_property(
            &self,
            key: $crate::value::propertykey::PropertyKey,
            value: $crate::value::object::PropertyValue,
            sc: &mut $crate::localscope::LocalScope,
        ) -> Result<(), $crate::value::Value> {
            self.$field.set_property(key, value, sc)
        }
    };
    (override $field:ident, delete_property) => {
        fn delete_property(
            &self,
            key: $crate::value::propertykey::PropertyKey,
            sc: &mut $crate::localscope::LocalScope,
        ) -> Result<$crate::value::Unrooted, $crate::value::Value> {
            self.$field.delete_property(key, sc)
        }
    };
    (override $field:ident, set_prototype) => {
        fn set_prototype(&self, value: $crate::value::Value, sc: &mut $crate::localscope::LocalScope) -> Result<(), $crate::value::Value> {
            self.$field.set_prototype(value, sc)
        }
    };
    (override $field:ident, get_prototype) => {
        fn get_prototype(&self, sc: &mut $crate::localscope::LocalScope) -> Result<$crate::value::Value, $crate::value::Value> {
            self.$field.get_prototype(sc)
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
            id: $crate::gc::ObjectId,
            this: $crate::frame::This,
            args: $crate::value::function::args::CallArgs,
            sc: &mut $crate::localscope::LocalScope,
        ) -> Result<$crate::value::Unrooted, $crate::value::Unrooted> {
            $crate::value::object::Object::apply(&self.$field, id, this, args, sc)
        }
    };
    (override $field:ident, construct) => {
        fn construct(
            &self,
            id: $crate::gc::ObjectId,
            this: $crate::frame::This,
            args: $crate::value::function::args::CallArgs,
            new_target: $crate::gc::ObjectId,
            sc: &mut $crate::localscope::LocalScope,
        ) -> Result<$crate::value::Unrooted, $crate::value::Unrooted> {
            $crate::value::object::Object::construct(&self.$field, id, this, args, new_target, sc)
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

/// An **ordinary** object (object with default behavior for the internal methods).
#[derive(Debug, Clone)]
pub struct OrdObject {
    prototype: RefCell<Option<ObjectId>>,
    constructor: RefCell<Option<ObjectId>>,
    values: RefCell<ObjectMap<PropertyKey, PropertyValue>>,
}

bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn get_or_apply(&self, sc: &mut LocalScope, this: This) -> Result<Unrooted, Unrooted> {
        self.kind.get_or_apply(sc, this)
    }

    pub fn to_descriptor_value(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        let obj = OrdObject::new(sc);

        match self.kind {
            PropertyValueKind::Static(value) => {
                obj.set_property(sym::value.to_key(sc), PropertyValue::static_default(value), sc)?;
            }
            PropertyValueKind::Trap { get, set } => {
                let get = get.map(Value::object).unwrap_or_undefined();
                let set = set.map(Value::object).unwrap_or_undefined();
                obj.set_property(sym::get.to_key(sc), PropertyValue::static_default(get), sc)?;
                obj.set_property(sym::set.to_key(sc), PropertyValue::static_default(set), sc)?;
            }
        }

        obj.set_property(
            sym::writable.to_key(sc),
            PropertyValue::static_default(Value::boolean(
                self.descriptor.contains(PropertyDataDescriptor::WRITABLE),
            )),
            sc,
        )?;

        obj.set_property(
            sym::enumerable.to_key(sc),
            PropertyValue::static_default(Value::boolean(
                self.descriptor.contains(PropertyDataDescriptor::ENUMERABLE),
            )),
            sc,
        )?;

        obj.set_property(
            sym::configurable.to_key(sc),
            PropertyValue::static_default(Value::boolean(
                self.descriptor.contains(PropertyDataDescriptor::CONFIGURABLE),
            )),
            sc,
        )?;

        Ok(Value::object(sc.register(obj)))
    }

    pub fn from_descriptor_value(sc: &mut LocalScope<'_>, value: Value) -> Result<Self, Value> {
        let mut flags = PropertyDataDescriptor::empty();
        let configurable = value
            .get_property(sym::configurable.to_key(sc), sc)
            .root(sc)?
            .into_option();
        let enumerable = value
            .get_property(sym::enumerable.to_key(sc), sc)
            .root(sc)?
            .into_option();
        let writable = value.get_property(sym::writable.to_key(sc), sc).root(sc)?.into_option();

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

        let static_value = value.get_property(sym::value.to_key(sc), sc).root(sc)?.into_option();
        let kind = match static_value {
            Some(static_value) => PropertyValueKind::Static(static_value),
            None => {
                let get = value
                    .get_property(sym::get.to_key(sc), sc)
                    .root(sc)?
                    .into_option()
                    .and_then(|v| match v.unpack() {
                        ValueKind::Object(o) => Some(o),
                        _ => None,
                    });
                let set = value
                    .get_property(sym::set.to_key(sc), sc)
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

    pub fn get_or_apply(&self, sc: &mut LocalScope, this: This) -> Result<Unrooted, Unrooted> {
        match *self {
            Self::Static(value) => Ok(value.into()),
            Self::Trap { get, .. } => match get {
                Some(id) => id.apply(this, CallArgs::empty(), sc),
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

impl OrdObject {
    pub fn new(vm: &Vm) -> Self {
        Self::with_values(vm, ObjectMap::default())
    }

    pub fn with_values(vm: &Vm, values: ObjectMap<PropertyKey, PropertyValue>) -> Self {
        Self {
            prototype: RefCell::new(Some(vm.statics.object_prototype)),
            constructor: RefCell::new(Some(vm.statics.object_ctor)), // TODO: function_ctor instead
            values: RefCell::new(values),
        }
    }

    /// Takes a constructor `new_target` and instantiates it
    pub fn instance_for_new_target(new_target: ObjectId, scope: &mut LocalScope) -> Result<Self, Value> {
        let ValueKind::Object(prototype) = new_target
            .get_property(sym::prototype.to_key(scope), scope)
            .root(scope)?
            .unpack()
        else {
            throw!(scope, Error, "new.target prototype must be an object")
        };
        Ok(Self::with_prototype_and_constructor(prototype, new_target))
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

unsafe impl Trace for OrdObject {
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

impl Object for OrdObject {
    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        match key.to_js_string(sc) {
            Some(sym::__proto__) => return Ok(Some(PropertyValue::static_default(self.get_prototype(sc)?))),
            Some(sym::constructor) => {
                return Ok(Some(PropertyValue::static_default(
                    self.constructor.borrow().map(Value::object).unwrap_or_undefined(),
                )));
            }
            _ => {}
        }

        let values = self.values.borrow();
        if let Some(value) = values.get(&key).cloned() {
            return Ok(Some(value));
        }

        Ok(None)
    }

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope) -> Result<(), Value> {
        match key.to_js_string(sc) {
            Some(sym::__proto__) => {
                return self.set_prototype(
                    match value.into_kind() {
                        PropertyValueKind::Static(value) => value,
                        _ => throw!(sc, TypeError, "Prototype cannot be a trap"),
                    },
                    sc,
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

    fn delete_property(&self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Value> {
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
        _handle: ObjectId,
        _this: This,
        _args: CallArgs,
        _sc: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        Ok(Value::undefined().into())
    }

    fn set_prototype(&self, value: Value, sc: &mut LocalScope) -> Result<(), Value> {
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

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        let values = self.values.borrow();
        Ok(values.keys().map(|k| k.to_value(sc)).collect())
    }

    extract!(self);
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
    fn get_own_property(&self, this: This, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        unsafe { (self.vtable(sc).js_get_own_property)(self.data_ptr(sc), this, key, sc) }
    }

    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        unsafe { (self.vtable(sc).js_get_own_property_descriptor)(self.data_ptr(sc), key, sc) }
    }

    fn get_property(&self, this: This, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        unsafe { (self.vtable(sc).js_get_property)(self.data_ptr(sc), this, key, sc) }
    }

    fn get_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        unsafe { (self.vtable(sc).js_get_property_descriptor)(self.data_ptr(sc), key, sc) }
    }

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope) -> Result<(), Value> {
        unsafe { (self.vtable(sc).js_set_property)(self.data_ptr(sc), key, value, sc) }
    }

    fn delete_property(&self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Value> {
        unsafe { (self.vtable(sc).js_delete_property)(self.data_ptr(sc), key, sc) }
    }

    fn set_prototype(&self, value: Value, sc: &mut LocalScope) -> Result<(), Value> {
        unsafe { (self.vtable(sc).js_set_prototype)(self.data_ptr(sc), value, sc) }
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        unsafe { (self.vtable(sc).js_get_prototype)(self.data_ptr(sc), sc) }
    }

    fn apply(
        &self,
        callee: ObjectId,
        this: This,
        args: CallArgs,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        unsafe { (self.vtable(scope).js_apply)(self.data_ptr(scope), callee, this, args, scope) }
    }

    fn construct(
        &self,
        callee: ObjectId,
        this: This,
        args: CallArgs,
        new_target: ObjectId,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        unsafe { (self.vtable(scope).js_construct)(self.data_ptr(scope), callee, this, args, new_target, scope) }
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

    fn extract_type_raw(&self, vm: &Vm, type_id: TypeId) -> Option<NonNull<()>> {
        unsafe { (self.vtable(vm).js_extract_type_raw)(self.data_ptr(vm), vm, type_id) }
    }
}

pub fn extract_type<'a, T: 'static>(v: &'a (impl Object + ?Sized), vm: &Vm) -> Option<&'a T> {
    let ptr = v.extract_type_raw(vm, TypeId::of::<T>())?;
    // SAFETY: `extract_type_raw` only returns `Some(_)` for types with the same TypeId
    Some(unsafe { ptr.cast().as_ref() })
}

impl ObjectId {
    pub fn extract<T: 'static>(&self, vm: &Vm) -> Option<&T> {
        extract_type::<T>(self, vm)
    }

    pub fn get_property(self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        Object::get_property(&self, This::Bound(Value::object(self)), key, sc)
    }

    pub fn get_own_property(self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        Object::get_own_property(&self, This::Bound(Value::object(self)), key, sc)
    }

    pub fn apply(&self, this: This, args: CallArgs, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        let callee = *self;
        Object::apply(self, callee, this, args, sc)
    }

    pub fn construct(&self, this: This, args: CallArgs, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        Object::construct(self, *self, this, args, *self, sc)
    }

    pub fn construct_with_target(
        &self,
        this: This,
        args: CallArgs,
        new_target: ObjectId,
        sc: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        Object::construct(self, *self, this, args, new_target, sc)
    }

    pub fn set_integrity_level(self, level: IntegrityLevel, sc: &mut LocalScope<'_>) -> Result<(), Value> {
        // TODO: invoke [[PreventExtensions]]
        let keys = self.own_keys(sc)?;
        for key in keys {
            let key = PropertyKey::from_value(sc, key)?;

            if let Some(mut desc) = self.get_own_property_descriptor(key, sc).root_err(sc)? {
                desc.descriptor.remove(PropertyDataDescriptor::CONFIGURABLE);
                if let IntegrityLevel::Frozen = level {
                    if let PropertyValueKind::Static(_) = desc.kind {
                        desc.descriptor.remove(PropertyDataDescriptor::WRITABLE);
                    }
                }
                self.set_property(key, desc, sc)?;
            }
        }
        Ok(())
    }
}

pub enum IntegrityLevel {
    Sealed,
    Frozen,
}

impl Persistent {
    pub fn get_property(&self, key: PropertyKey, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        self.id().get_property(key, sc)
    }

    pub fn apply(&self, this: This, args: CallArgs, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        self.id().apply(this, args, sc)
    }

    pub fn construct(&self, this: This, args: CallArgs, sc: &mut LocalScope) -> Result<Unrooted, Unrooted> {
        self.id().construct(this, args, sc)
    }

    // FIXME: should override typeof, internal_slots, etc.
}

/// Delegates a get_property call to get_property_descriptor and converts the return value respectively
pub fn delegate_get_property<T: Object + ?Sized>(
    this: &T,
    this_value: This,
    sc: &mut LocalScope,
    key: PropertyKey,
) -> Result<Unrooted, Unrooted> {
    this.get_property_descriptor(key, sc)
        .map(|x| x.unwrap_or_else(|| PropertyValue::static_default(Value::undefined())))
        .and_then(|x| x.get_or_apply(sc, this_value))
}
/// Delegates a get_property call to get_property_descriptor and converts the return value respectively
pub fn delegate_get_own_property<T: Object + ?Sized>(
    this: &T,
    this_value: This,
    sc: &mut LocalScope,
    key: PropertyKey,
) -> Result<Unrooted, Unrooted> {
    this.get_own_property_descriptor(key, sc)
        .map(|x| x.unwrap_or_else(|| PropertyValue::static_default(Value::undefined())))
        .and_then(|x| x.get_or_apply(sc, this_value))
}
