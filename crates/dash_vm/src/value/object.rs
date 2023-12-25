use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ptr::addr_of;

use crate::gc::persistent::Persistent;
use crate::gc::trace::{Trace, TraceCtxt};
use bitflags::bitflags;
use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::localscope::LocalScope;
use crate::{throw, Vm};

use super::ops::conversions::ValueConversion;
use super::primitive::{PrimitiveCapabilities, Symbol};
use super::{ExternalValue, Root, Typeof, Unrooted, Value, ValueContext};

pub type ObjectMap<K, V> = ahash::HashMap<K, V>;

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

        match self.get_prototype(sc)? {
            Value::Object(object) => object.get_property_descriptor(sc, key),
            Value::External(object) => object.get_own_property_descriptor(sc, key),
            Value::Null(..) => Ok(None),
            _ => unreachable!(),
        }
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value>;

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value>;

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value>;

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value>;

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted>;

    fn construct(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any;

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        None
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value>;

    fn type_of(&self) -> Typeof {
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
            key: $crate::value::object::PropertyKey<'static>,
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
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    };
    (override $field:ident, own_keys) => {
        fn own_keys(&self) -> Result<Vec<$crate::value::Value>, $crate::value::Value> {
            self.$field.own_keys()
        }
    };
    (override $field:ident, apply) => {
        fn apply(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            handle: $crate::gc::handle::Handle<dyn Object>,
            this: $crate::value::Value,
            args: Vec<$crate::value::Value>,
        ) -> Result<$crate::value::Unrooted, $crate::value::Unrooted> {
            $crate::value::object::Object::apply(&self.$field, sc, handle, this, args)
        }
    };
    (override $field:ident, construct) => {
        fn construct(
            &self,
            sc: &mut $crate::localscope::LocalScope,
            handle: $crate::gc::handle::Handle<dyn Object>,
            this: $crate::value::Value,
            args: Vec<$crate::value::Value>,
        ) -> Result<$crate::value::Unrooted, $crate::value::Unrooted> {
            $crate::value::object::Object::construct(&self.$field, sc, handle, this, args)
        }
    };
    (override $field:ident, type_of) => {
        fn type_of(&self) -> $crate::value::Typeof {
            self.$field.type_of()
        }
    };
    (override $field:ident, as_primitive_capable) => {
        fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
            self.$field.as_primitive_capable()
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
    prototype: RefCell<Option<Handle<dyn Object>>>,
    constructor: RefCell<Option<Handle<dyn Object>>>,
    values: RefCell<ObjectMap<PropertyKey<'static>, PropertyValue>>,
}

// TODO: optimization opportunity: some kind of Number variant for faster indexing without .to_string()
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PropertyKey<'a> {
    String(Cow<'a, str>),
    Symbol(Symbol),
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

#[derive(Debug, Clone, Trace)]
pub struct PropertyValue {
    kind: PropertyValueKind,
    descriptor: PropertyDataDescriptor,
}

impl PropertyValue {
    pub fn new(kind: PropertyValueKind, descriptor: PropertyDataDescriptor) -> Self {
        Self { kind, descriptor }
    }

    /// Convenience function for creating a static property with a default descriptor (all bits set to 1)
    pub fn static_default(value: Value) -> Self {
        Self::new(PropertyValueKind::Static(value), Default::default())
    }

    pub fn getter_default(value: Handle<dyn Object>) -> Self {
        Self::new(
            PropertyValueKind::Trap {
                get: Some(value),
                set: None,
            },
            Default::default(),
        )
    }

    pub fn setter_default(value: Handle<dyn Object>) -> Self {
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
                obj.set_property(sc, "value".into(), PropertyValue::static_default(value.clone()))?;
            }
            PropertyValueKind::Trap { get, set } => {
                let get = get.as_ref().map(|v| Value::Object(v.clone())).unwrap_or_undefined();
                let set = set.as_ref().map(|v| Value::Object(v.clone())).unwrap_or_undefined();
                obj.set_property(sc, "get".into(), PropertyValue::static_default(get))?;
                obj.set_property(sc, "set".into(), PropertyValue::static_default(set))?;
            }
        }

        obj.set_property(
            sc,
            "writable".into(),
            PropertyValue::static_default(Value::Boolean(
                self.descriptor.contains(PropertyDataDescriptor::WRITABLE),
            )),
        )?;

        obj.set_property(
            sc,
            "enumerable".into(),
            PropertyValue::static_default(Value::Boolean(
                self.descriptor.contains(PropertyDataDescriptor::ENUMERABLE),
            )),
        )?;

        obj.set_property(
            sc,
            "configurable".into(),
            PropertyValue::static_default(Value::Boolean(
                self.descriptor.contains(PropertyDataDescriptor::CONFIGURABLE),
            )),
        )?;

        Ok(Value::Object(sc.register(obj)))
    }

    pub fn from_descriptor_value(sc: &mut LocalScope<'_>, value: Value) -> Result<Self, Value> {
        let mut flags = PropertyDataDescriptor::empty();
        let configurable = value.get_property(sc, "configurable".into()).root(sc)?.into_option();
        let enumerable = value.get_property(sc, "enumerable".into()).root(sc)?.into_option();
        let writable = value.get_property(sc, "writable".into()).root(sc)?.into_option();

        if configurable.map_or(true, |v| v.is_truthy()) {
            flags |= PropertyDataDescriptor::CONFIGURABLE;
        }
        if enumerable.map_or(true, |v| v.is_truthy()) {
            flags |= PropertyDataDescriptor::ENUMERABLE;
        }
        if writable.map_or(true, |v| v.is_truthy()) {
            flags |= PropertyDataDescriptor::WRITABLE;
        }

        // TODO: make sure that if value is set, get/set are not

        let static_value = value.get_property(sc, "value".into()).root(sc)?.into_option();
        let kind = match static_value {
            Some(static_value) => PropertyValueKind::Static(static_value),
            None => {
                let get = value
                    .get_property(sc, "get".into())
                    .root(sc)?
                    .into_option()
                    .and_then(|v| match v {
                        Value::Object(o) => Some(o),
                        _ => None,
                    });
                let set = value
                    .get_property(sc, "set".into())
                    .root(sc)?
                    .into_option()
                    .and_then(|v| match v {
                        Value::Object(o) => Some(o),
                        _ => None,
                    });

                PropertyValueKind::Trap { get, set }
            }
        };

        Ok(Self::new(kind, flags))
    }
}

// TODO: these handles should be "hidden" behind something similar to the `Unrooted` value
#[derive(Debug, Clone)]
pub enum PropertyValueKind {
    /// Accessor property
    Trap {
        get: Option<Handle<dyn Object>>,
        set: Option<Handle<dyn Object>>,
    },
    /// Static value property
    Static(Value),
    // TODO: magic property that appears "static" but is actually computed, e.g. array.length
}

impl PropertyValueKind {
    pub fn getter(get: Handle<dyn Object>) -> Self {
        Self::Trap {
            get: Some(get),
            set: None,
        }
    }

    pub fn setter(set: Handle<dyn Object>) -> Self {
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
                Some(handle) => handle.apply(sc, this, Vec::new()),
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

impl<'a> PropertyKey<'a> {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyKey::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }
}

impl<'a> From<&'a str> for PropertyKey<'a> {
    fn from(s: &'a str) -> Self {
        PropertyKey::String(Cow::Borrowed(s))
    }
}

impl From<String> for PropertyKey<'static> {
    fn from(s: String) -> Self {
        PropertyKey::String(Cow::Owned(s))
    }
}

impl From<Symbol> for PropertyKey<'static> {
    fn from(s: Symbol) -> Self {
        PropertyKey::Symbol(s)
    }
}

impl<'a> PropertyKey<'a> {
    pub fn as_value(&self) -> Value {
        match self {
            PropertyKey::String(s) => Value::String(s.as_ref().into()),
            PropertyKey::Symbol(s) => Value::Symbol(s.clone()),
        }
    }

    pub fn from_value(sc: &mut LocalScope, value: Value) -> Result<Self, Value> {
        match value {
            Value::Symbol(s) => Ok(Self::Symbol(s)),
            other => {
                let key = other.to_string(sc)?;
                Ok(PropertyKey::String(ToString::to_string(&key).into()))
            }
        }
    }
}

impl NamedObject {
    pub fn new(vm: &Vm) -> Self {
        Self::with_values(vm, ObjectMap::default())
    }

    pub fn with_values(vm: &Vm, values: ObjectMap<PropertyKey<'static>, PropertyValue>) -> Self {
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

    pub fn with_prototype_and_constructor(prototype: Handle<dyn Object>, ctor: Handle<dyn Object>) -> Self {
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
        let values = self.values.borrow();
        for value in values.values() {
            value.trace(cx);
        }

        let prototype = self.prototype.borrow();
        if let Some(prototype) = &*prototype {
            prototype.trace(cx);
        }

        let constructor = self.constructor.borrow();
        if let Some(constructor) = &*constructor {
            constructor.trace(cx);
        }
    }
}

impl Object for NamedObject {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        if let PropertyKey::String(st) = &key {
            match st.as_ref() {
                "__proto__" => return Ok(Some(PropertyValue::static_default(self.get_prototype(sc)?))),
                "constructor" => {
                    return Ok(Some(PropertyValue::static_default(
                        self.constructor
                            .borrow()
                            .as_ref()
                            .map(|x| Value::from(x.clone()))
                            .unwrap_or_undefined(),
                    )));
                }
                _ => {}
            }
        };

        let values = self.values.borrow();
        if let Some(value) = values.get(&key).cloned() {
            return Ok(Some(value));
        }

        // if let Some(prototype) = self.prototype.borrow().as_ref() {
        //     return prototype.get_property_descriptor(sc, key);
        // }

        Ok(None)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        match key.as_string() {
            Some("__proto__") => {
                return self.set_prototype(
                    sc,
                    match value.into_kind() {
                        PropertyValueKind::Static(value) => value,
                        _ => throw!(sc, TypeError, "Prototype cannot be a trap"),
                    },
                );
            }
            Some("constructor") => {
                let obj = match value.kind {
                    PropertyValueKind::Static(Value::Object(obj)) => obj,
                    PropertyValueKind::Static(Value::External(obj)) => obj.inner.clone(),
                    _ => throw!(sc, TypeError, "constructor is not an object"), // TODO: it doesn't need to be
                };
                self.constructor.replace(Some(obj));
                return Ok(());
            }
            _ => {}
        };

        // TODO: check if we are invoking a setter

        let mut map = self.values.borrow_mut();
        map.insert(key, value);
        Ok(())
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        let key = unsafe { &*addr_of!(key).cast::<PropertyKey<'static>>() };

        let mut values = self.values.borrow_mut();
        let value = values.remove(key);

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
        _handle: Handle<dyn Object>,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        Ok(Value::undefined().into())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        match value {
            Value::Null(_) => self.prototype.replace(None),
            Value::Object(handle) => self.prototype.replace(Some(handle)),
            Value::External(handle) => self.prototype.replace(Some(handle.inner.clone())), // TODO: check that handle is an object
            _ => throw!(sc, TypeError, "prototype must be an object"),
        };

        Ok(())
    }

    fn get_prototype(&self, _sc: &mut LocalScope) -> Result<Value, Value> {
        let prototype = self.prototype.borrow();
        match prototype.as_ref() {
            Some(handle) => Ok(Value::Object(handle.clone())),
            None => Ok(Value::null()),
        }
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        let values = self.values.borrow();
        Ok(values.keys().map(PropertyKey::as_value).collect())
    }
}

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

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
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
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        (**self).apply(scope, callee, this, args)
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        (**self).construct(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        (**self).own_keys()
    }

    fn type_of(&self) -> Typeof {
        (**self).type_of()
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        (**self).as_primitive_capable()
    }
}

impl Object for Handle<dyn Object> {
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

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
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
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        (**self).apply(scope, callee, this, args)
    }

    fn construct(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        (**self).construct(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        (**self).own_keys()
    }

    fn type_of(&self) -> Typeof {
        (**self).type_of()
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        (**self).as_primitive_capable()
    }
}

impl Handle<ExternalValue> {
    pub fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        self.inner.get_property(sc, key)
    }

    pub fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        self.inner.apply(sc, this, args)
    }

    pub fn construct(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        self.inner.construct(sc, this, args)
    }

    // FIXME: should override typeof, as_primitive_capable, etc.
}

impl Handle<dyn Object> {
    pub fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        (**self).get_property(sc, Value::Object(self.clone()), key)
    }

    pub fn get_own_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        (**self).get_own_property(sc, Value::Object(self.clone()), key)
    }

    pub fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        let callee = self.clone();
        (**self).apply(sc, callee, this, args)
    }

    pub fn construct(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        let callee = self.clone();
        (**self).construct(sc, callee, this, args)
    }

    // FIXME: should override typeof, as_primitive_capable, etc.
}

impl Persistent<dyn Object> {
    pub fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Unrooted> {
        let callee = self.handle().clone();
        (**self).get_property(sc, Value::Object(callee), key)
    }

    pub fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        let callee = self.handle().clone();
        (**self).apply(sc, callee, this, args)
    }

    pub fn construct(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Unrooted, Unrooted> {
        let callee = self.handle().clone();
        (**self).construct(sc, callee, this, args)
    }

    // FIXME: should override typeof, as_primitive_capable, etc.
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
