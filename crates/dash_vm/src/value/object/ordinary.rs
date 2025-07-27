use std::alloc::Layout;
use std::fmt::Debug;
use std::iter;
use std::mem::MaybeUninit;
use std::num::NonZero;
use std::ptr::{self, NonNull};
use std::sync::LazyLock;

use dash_middle::interner::{self};
use dash_middle::unsaferefcell::UnsafeRefCell;

use crate::gc::ObjectId;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::localscope::LocalScope;
use crate::util::cold_path;
use crate::value::function::args::CallArgs;
use crate::value::object::This;
use crate::value::primitive::Symbol;
use crate::value::propertykey::{PropertyKey, PropertyKeyInner, ToPropertyKey};
use crate::value::string::JsString;
use crate::value::{Root, Unpack, Unrooted, Value, ValueKind};
use crate::{Vm, extract, throw};

use super::{Object, PropertyDataDescriptor, PropertyValue, PropertyValueKind};

#[derive(Copy, Clone)]
struct X86Features {
    sse2: bool,
    avx2: bool,
}

static X86_FEATURES: LazyLock<X86Features> = LazyLock::new(|| X86Features {
    sse2: is_x86_feature_detected!("sse2"),
    avx2: is_x86_feature_detected!("avx2"),
});

#[derive(Debug)]
enum InnerOrdObject {
    // TODO: prototype should just be Option<ObjectId>..?
    Cow { prototype: PropertyValue },
    Linear(PropertyVec),
}

/// An **ordinary** object (object with default behavior for the internal methods).
#[derive(Debug)]
pub struct OrdObject(UnsafeRefCell<InnerOrdObject>);

unsafe impl Trace for OrdObject {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        // SAFETY: we may re-enter during tracing, but we only immutable borrow it ever so that's fine.
        // none of the methods calling borrow_mut() can be reached during tracing since they require a &mut LocalScope
        let cell = &*unsafe { self.0.borrow() };

        match cell {
            InnerOrdObject::Cow { prototype } => prototype.trace(cx),
            InnerOrdObject::Linear(property_vec) => {
                let PropertyVecAllocation {
                    len: _,
                    cap: _,
                    string_key_count: _,
                    symbol_key_count: _,
                    prototype,
                    data: _,
                } = unsafe { &*property_vec.0.as_ptr() };

                prototype.trace(cx);

                property_vec
                    .string_keys()
                    .iter()
                    .chain(property_vec.symbol_keys())
                    .copied()
                    .for_each(|sym| {
                        cx.mark_symbol(interner::Symbol::from_raw(sym));
                    });

                let (values, descriptors) = property_vec.values_descriptors();

                for (&value, &descriptor) in iter::zip(values, descriptors) {
                    if descriptor.contains(InternalLinearPropertyVecDescriptor::GET) {
                        // SAFETY: descriptor contains `GET`, so `value.trap.get` is initialized
                        let get = unsafe { value.trap.get.assume_init() };
                        get.trace(cx);
                    }

                    if descriptor.contains(InternalLinearPropertyVecDescriptor::SET) {
                        // SAFETY: descriptor contains `SET`, so `value.trap.set` is initialized
                        let set = unsafe { value.trap.set.assume_init() };
                        set.trace(cx);
                    }

                    if !descriptor.intersects(InternalLinearPropertyVecDescriptor::GET_SET) {
                        // SAFETY: descriptor contains neither `GET` nor `SET`, so this is a `value.static_`
                        let value = unsafe { value.static_ };
                        value.trace(cx);
                    }
                }
            }
        }
    }
}

impl OrdObject {
    pub fn new(vm: &Vm) -> Self {
        Self(UnsafeRefCell::new(InnerOrdObject::Cow {
            prototype: PropertyValue::static_default(Value::object(vm.statics.object_prototype)),
        }))
    }

    pub fn with_prototype(prototype: ObjectId) -> Self {
        Self(UnsafeRefCell::new(InnerOrdObject::Cow {
            prototype: PropertyValue::static_default(prototype.into()),
        }))
    }

    pub fn with_prototype_and_ctor(prototype: ObjectId, ctor: ObjectId) -> Self {
        let mut pvec = PropertyVec::new(PropertyValue::static_default(prototype.into()));
        pvec.set_property(PropertyKey::CONSTRUCTOR, PropertyValue::static_default(ctor.into()));
        Self(UnsafeRefCell::new(InnerOrdObject::Linear(pvec)))
    }

    pub fn null() -> Self {
        Self(UnsafeRefCell::new(InnerOrdObject::Cow {
            prototype: PropertyValue::static_default(Value::null()),
        }))
    }

    /// Takes a constructor `new_target` and instantiates it
    pub fn instance_for_new_target(new_target: ObjectId, scope: &mut LocalScope) -> Result<Self, Value> {
        let ValueKind::Object(prototype) = new_target
            .get_property(interner::sym::prototype.to_key(scope), scope)
            .root(scope)?
            .unpack()
        else {
            throw!(scope, Error, "new.target prototype must be an object")
        };
        Ok(Self::with_prototype(prototype))
    }

    pub fn dump(&self, sc: &mut LocalScope<'_>) -> Result<(), Unrooted> {
        for key in self.own_keys(sc)? {
            let value = self.get_property(This::default(), PropertyKey::from_value(sc, key)?, sc)?;
            eprintln!("{:?} -> {:?}", key.unpack(), value.root(sc).unpack());
        }
        Ok(())
    }
}

#[inline(always)] // Very hot, forcing inline reduces runtime in some lookup microbenchmarks by up to 60%
fn get_own_property_descriptor_inline(
    object: &InnerOrdObject,
    key: PropertyKey,
) -> Result<Option<PropertyValue>, Unrooted> {
    // TODO: stop handling __proto__ here and do that in the handler instead?

    match *object {
        InnerOrdObject::Cow { prototype } => {
            if key == PropertyKey::PROTO {
                Ok(Some(prototype))
            } else {
                Ok(None)
            }
        }
        InnerOrdObject::Linear(ref property_vec) => {
            if key == PropertyKey::PROTO {
                Ok(Some(property_vec.get_prototype()))
            } else {
                Ok(property_vec.get_property(key))
            }
        }
    }
}

impl Object for OrdObject {
    #[inline(always)]
    fn get_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        // SAFETY: no reentrancy possible from here
        let cell = unsafe { self.0.borrow() };

        match get_own_property_descriptor_inline(&cell, key) {
            Ok(Some(v)) => Ok(Some(v)),
            Ok(None) => {
                cold_path();
                drop(cell);

                match self.get_prototype(sc)?.unpack() {
                    ValueKind::Object(object) => object.get_property_descriptor(key, sc),
                    ValueKind::External(object) => object.get_own_property_descriptor(key, sc),
                    ValueKind::Null(..) => Ok(None),
                    _ => unreachable!(),
                }
            }
            Err(err) => {
                cold_path();
                Err(err)
            }
        }
    }

    #[inline(always)]
    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        _: &mut LocalScope<'_>,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        // SAFETY: no reentrancy possible from here
        let cell = unsafe { self.0.borrow() };
        get_own_property_descriptor_inline(&cell, key)
    }

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope<'_>) -> Result<(), Value> {
        // TODO: stop handling __proto__ here and do that in the handler instead? the way we do that here is kinda wrong.
        // SAFETY: no reentrancy possible from here
        let mut guard = unsafe { self.0.borrow_mut() };

        match &mut *guard {
            InnerOrdObject::Cow { prototype } => {
                if key == PropertyKey::PROTO {
                    *prototype = value;
                    Ok(())
                } else {
                    let prototype = *prototype;
                    *guard = InnerOrdObject::Linear(PropertyVec::new(prototype));
                    drop(guard);
                    self.set_property(key, value, sc)
                }
            }
            InnerOrdObject::Linear(property_vec) => {
                if key == PropertyKey::PROTO {
                    *property_vec.get_prototype_mut() = value;
                    Ok(())
                } else {
                    match property_vec.set_property(key, value) {
                        SetPropertyResult::Ok => Ok(()),
                        // TODO: throw in strict mode?
                        SetPropertyResult::NotWritable => {
                            cold_path();
                            Ok(())
                        }
                        SetPropertyResult::InvokeSetter(alloc_id) => {
                            drop(guard);

                            if let PropertyValueKind::Static(value) = value.kind() {
                                match alloc_id.apply(This::default(), [*value].into(), sc) {
                                    Ok(_) => Ok(()),
                                    Err(err) => Err(err.root(sc)),
                                }
                            } else {
                                Ok(())
                            }
                        }
                    }
                }
            }
        }
    }

    fn delete_property(&self, key: PropertyKey, sc: &mut LocalScope<'_>) -> Result<Unrooted, Value> {
        // SAFETY: no reentrancy possible from here
        let cell = unsafe { &mut *self.0.borrow_mut() };

        match cell {
            InnerOrdObject::Cow { prototype: _ } => Ok(Value::undefined().into()),
            InnerOrdObject::Linear(property_vec) => match property_vec.delete_property(key) {
                Some(pv) => match pv.kind() {
                    PropertyValueKind::Static(v) => {
                        sc.add_value(*v);
                        Ok((*v).into())
                    }
                    PropertyValueKind::Trap { get, set } => {
                        if let Some(get) = get {
                            sc.add_ref(*get);
                        }
                        if let Some(set) = set {
                            sc.add_ref(*set);
                        }
                        Ok(Value::undefined().into())
                    }
                },
                None => Ok(Value::undefined().into()),
            },
        }
    }

    fn set_prototype(&self, value: Value, _: &mut LocalScope<'_>) -> Result<(), Value> {
        // SAFETY: no reentrancy possible from here
        let cell = unsafe { &mut *self.0.borrow_mut() };

        match cell {
            InnerOrdObject::Cow { prototype } => {
                *prototype = PropertyValue::static_default(value);
                Ok(())
            }
            InnerOrdObject::Linear(property_vec) => {
                *property_vec.get_prototype_mut() = PropertyValue::static_default(value);
                Ok(())
            }
        }
    }

    fn get_prototype(&self, sc: &mut LocalScope<'_>) -> Result<Value, Value> {
        // SAFETY: no reentrancy possible from here
        let cell = unsafe { self.0.borrow() };

        match *cell {
            InnerOrdObject::Cow { prototype } => {
                drop(cell);
                prototype.get_or_apply(sc, This::default()).root(sc)
            }
            InnerOrdObject::Linear(ref property_vec) => {
                let prototype = property_vec.get_prototype();
                drop(cell);
                prototype.get_or_apply(sc, This::default()).root(sc)
            }
        }
    }

    fn apply(&self, _: ObjectId, _: This, _: CallArgs, scope: &mut LocalScope<'_>) -> Result<Unrooted, Unrooted> {
        throw!(scope, Error, "Attempted to call non-function object")
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        // SAFETY: no reentrancy possible from here
        let cell = unsafe { &*self.0.borrow() };

        match *cell {
            InnerOrdObject::Cow { prototype: _ } => Ok(Vec::new()),
            InnerOrdObject::Linear(ref property_vec) => {
                let mut keys = Vec::with_capacity(property_vec.raw_keys().len());

                for &sym in property_vec.string_keys() {
                    keys.push(Value::string(JsString::from_sym(interner::Symbol::from_raw(sym))));
                }

                for &sym in property_vec.symbol_keys() {
                    keys.push(Value::symbol(Symbol::new(JsString::from_sym(
                        interner::Symbol::from_raw(sym),
                    ))));
                }

                for &index in property_vec.index_keys() {
                    keys.push(Value::number(index as f64));
                }

                Ok(keys)
            }
        }
    }

    extract!(self);
}

bitflags::bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct InternalLinearPropertyVecDescriptor: u8 {
        const CONFIGURABLE = 1 << 0;
        const ENUMERABLE = 1 << 1;
        const WRITABLE = 1 << 2;
        const GET = 1 << 3;
        const SET = 1 << 4;
    }
}

impl InternalLinearPropertyVecDescriptor {
    const GET_SET: Self = Self::union(Self::GET, Self::SET);
}

#[derive(Copy, Clone)]
struct InternalLinearPropertyVecTrap {
    // Initialized if `InternalLinearPropertyVecDescriptor` contains `GET`
    get: MaybeUninit<ObjectId>,
    // Initialized if `InternalLinearPropertyVecDescriptor` contains `SET`
    set: MaybeUninit<ObjectId>,
}

// discriminant indicated by `InternalLinearPropertyVecDescriptor`
#[derive(Copy, Clone)]
union InternalLinearPropertyVecValue {
    trap: InternalLinearPropertyVecTrap,
    static_: Value,
}

fn property_value_to_internal(
    value: PropertyValue,
) -> (InternalLinearPropertyVecValue, InternalLinearPropertyVecDescriptor) {
    let mut descriptor = InternalLinearPropertyVecDescriptor::empty();
    if value.descriptor.contains(PropertyDataDescriptor::CONFIGURABLE) {
        descriptor |= InternalLinearPropertyVecDescriptor::CONFIGURABLE;
    }
    if value.descriptor.contains(PropertyDataDescriptor::ENUMERABLE) {
        descriptor |= InternalLinearPropertyVecDescriptor::ENUMERABLE;
    }
    if value.descriptor.contains(PropertyDataDescriptor::WRITABLE) {
        descriptor |= InternalLinearPropertyVecDescriptor::WRITABLE;
    }

    let value = match value.kind {
        PropertyValueKind::Trap { get, set } => {
            let get = match get {
                Some(get) => {
                    descriptor |= InternalLinearPropertyVecDescriptor::GET;
                    MaybeUninit::new(get)
                }
                None => MaybeUninit::uninit(),
            };

            let set = match set {
                Some(set) => {
                    descriptor |= InternalLinearPropertyVecDescriptor::SET;
                    MaybeUninit::new(set)
                }
                None => MaybeUninit::uninit(),
            };

            InternalLinearPropertyVecValue {
                trap: InternalLinearPropertyVecTrap { get, set },
            }
        }
        PropertyValueKind::Static(value) => InternalLinearPropertyVecValue { static_: value },
    };

    (value, descriptor)
}

unsafe fn align_ptr<T>(ptr: *const T) -> *const T {
    let extra_align = ptr.cast::<u8>().align_offset(align_of::<T>());
    unsafe { ptr.byte_add(extra_align) }
}

unsafe fn align_ptr_mut<T>(ptr: *mut T) -> *mut T {
    let extra_align = ptr.cast::<u8>().align_offset(align_of::<T>());
    unsafe { ptr.byte_add(extra_align) }
}

#[derive(Debug)]
struct PropertyVec(NonNull<PropertyVecAllocation>);

enum SetPropertyResult {
    Ok,
    NotWritable,
    InvokeSetter(ObjectId),
}

impl PropertyVec {
    const INITIAL_CAP: NonZero<u32> = NonZero::new(4).unwrap();

    const fn layout(cap: NonZero<u32>) -> Layout {
        // const compatible unwrap
        macro_rules! unwrap {
            ($x:expr) => {
                match $x {
                    Ok(v) => v,
                    Err(_) => panic!("failed to create layout"),
                }
            };
        }

        let base_layout = Layout::new::<PropertyVecAllocation>();
        let keys_layout = unwrap!(Layout::array::<interner::Symbol>(cap.get() as usize));

        let values_layout = unwrap!(Layout::array::<InternalLinearPropertyVecValue>(cap.get() as usize));
        let descriptor_layout = unwrap!(Layout::array::<InternalLinearPropertyVecDescriptor>(cap.get() as usize));

        let (layout, offset) = unwrap!(base_layout.extend(keys_layout));
        assert!(offset == size_of::<PropertyVecAllocation>()); // it should not need padding, as the struct has alignment of 4

        let (layout, _) = unwrap!(layout.extend(values_layout));
        let (layout, _) = unwrap!(layout.extend(descriptor_layout));

        assert!(layout.size() > 0);
        layout
    }

    /// Creates a linear property vector. This *always* allocates, even if no properties are stored.
    pub fn new(prototype: PropertyValue) -> Self {
        let layout = const { Self::layout(Self::INITIAL_CAP) };

        // SAFETY: `Self::layout` returns a valid, non zero sized layout
        let ptr = NonNull::new(unsafe { std::alloc::alloc(layout) })
            .unwrap_or_else(|| std::alloc::handle_alloc_error(layout))
            .cast::<PropertyVecAllocation>();

        // SAFETY: we just allocated the pointer, so it's valid for writes
        unsafe {
            ptr.write(PropertyVecAllocation {
                cap: Self::INITIAL_CAP,
                len: 0,
                data: [Aligned4Zst; 0],
                prototype,
                string_key_count: 0,
                symbol_key_count: 0,
            });
        }

        Self(ptr)
    }

    /// Ensures that there is enough capacity for `additional` more properties.
    /// This either reallocates and moves things around such that there is space for at least `additional` more properties,
    /// or simply bumps the capacity if there is enough.
    fn ensure_additional_properties(&mut self, additional: NonZero<u32>) {
        let mut required_cap = self.len().checked_add(additional.get()).expect("cap overflow");
        if required_cap > self.capacity().get() {
            required_cap = required_cap.next_power_of_two();

            // Realloc.
            let old_ptr = self.0.as_ptr();
            let old_layout = Self::layout(self.capacity());
            let new_layout = Self::layout(NonZero::new(required_cap).unwrap());

            let old_keys_ptr = self.data().cast::<u32>();
            let old_values_ptr = self.values_start();
            let old_descriptors_ptr = self.descriptors_start_mut();
            let len = self.len() as usize;

            // TODO: careful with panic safety

            // SAFETY: `Self::layout` returns a valid, non zero sized layout
            let new_ptr = NonNull::new(unsafe { std::alloc::alloc(new_layout) })
                .unwrap_or_else(|| std::alloc::handle_alloc_error(new_layout))
                .cast::<PropertyVecAllocation>();

            unsafe {
                new_ptr.write(PropertyVecAllocation {
                    cap: NonZero::new(required_cap).unwrap(),
                    len: (*old_ptr).len,
                    data: [Aligned4Zst; 0],
                    prototype: (*old_ptr).prototype,
                    string_key_count: (*old_ptr).string_key_count,
                    symbol_key_count: (*old_ptr).symbol_key_count,
                });

                self.0 = new_ptr;
                let new_keys_ptr = self.data_mut().cast::<u32>();
                let new_values_ptr = self.values_start_mut();
                let new_descriptors_ptr = self.descriptors_start_mut();
                ptr::copy_nonoverlapping(old_keys_ptr, new_keys_ptr, len);
                ptr::copy_nonoverlapping(old_values_ptr, new_values_ptr, len);
                ptr::copy_nonoverlapping(old_descriptors_ptr, new_descriptors_ptr, len);
            }

            // Finally deallocate the old pointer.
            unsafe { std::alloc::dealloc(old_ptr.cast(), old_layout) };
        }
    }

    pub fn set_property(&mut self, key: PropertyKey, value: PropertyValue) -> SetPropertyResult {
        if let Some(idx) = self.find_key_index(key) {
            let (values, descriptors) = self.values_descriptors_mut();

            let (val, descr) = property_value_to_internal(value);

            let value = &mut values[idx as usize];
            let descriptor = &mut descriptors[idx as usize];

            if !descriptor.contains(InternalLinearPropertyVecDescriptor::WRITABLE) {
                return SetPropertyResult::NotWritable;
            }

            if descriptor.contains(InternalLinearPropertyVecDescriptor::GET_SET) {
                cold_path();
                if descr.contains(InternalLinearPropertyVecDescriptor::GET) {
                    unsafe {
                        value.trap.get = val.trap.get;
                    }
                }
                if descr.contains(InternalLinearPropertyVecDescriptor::SET) {
                    unsafe {
                        value.trap.set = val.trap.set;
                    }
                }
                if descriptor.contains(InternalLinearPropertyVecDescriptor::SET) {
                    // SAFETY: descriptor contains `SET`, so `value.trap.set` is initialized
                    let set = unsafe { value.trap.set.assume_init() };
                    return SetPropertyResult::InvokeSetter(set);
                }
            } else {
                *value = val;
            }

            *descriptor = descr;

            SetPropertyResult::Ok
        } else {
            // Property does not exist. Define it.

            self.ensure_additional_properties(const { NonZero::new(1).unwrap() });

            // There's space for at least one more key at the very end now.
            // Depending on whether we want to insert a string, symbol or index key we may need
            // to shift parts of the keys section 1 to the right, so that for
            // TTTYYIUU     (T = string, Y = symbol, I = index, U = uninitialized/spare capacity)
            // Inserting a string becomes:  TTTUYYIU
            //                                 ^ here is where we'll put the key (`insert_at_index` before shifting)
            // Inserting a symbol becomes:  TTTYYUIU
            // Inserting an index becomes:  TTTYYIUU
            // We `.rotate_right(1)` on a MaybeUninit slice of the next keys section to the very end of the keys section + 1
            // We also need to do the same rotating for the other value/descriptor sections
            let (insert_at_index, raw_key) = match key.inner() {
                PropertyKeyInner::String(js_string) => (self.symbol_key_index_offset(), js_string.sym().raw()),
                PropertyKeyInner::Symbol(symbol) => (self.index_key_index_offset(), symbol.sym().raw()),
                PropertyKeyInner::Index(idx) => (self.index_key_index_offset() + self.index_key_count(), idx),
            };

            let len = self.len() as usize;

            fn insert_section<T>(ptr: *mut MaybeUninit<T>, section_len: usize, insert_at_index: u32, value: T) {
                let section =
                    unsafe { &mut std::slice::from_raw_parts_mut(ptr, section_len + 1)[insert_at_index as usize..] };

                section.rotate_right(1);
                section[0].write(value);
            }

            insert_section(
                self.data_mut().cast::<MaybeUninit<u32>>(),
                len,
                insert_at_index,
                raw_key,
            );

            let (value, descriptor) = property_value_to_internal(value);

            insert_section(
                self.values_start_mut()
                    .cast::<MaybeUninit<InternalLinearPropertyVecValue>>(),
                len,
                insert_at_index,
                value,
            );

            insert_section(
                self.descriptors_start_mut()
                    .cast::<MaybeUninit<InternalLinearPropertyVecDescriptor>>(),
                len,
                insert_at_index,
                descriptor,
            );

            unsafe { (*self.0.as_ptr()).len += 1 };
            match key.inner() {
                PropertyKeyInner::String(_) => {
                    unsafe { (*self.0.as_ptr()).string_key_count += 1 };
                }
                PropertyKeyInner::Symbol(_) => {
                    unsafe { (*self.0.as_ptr()).symbol_key_count += 1 };
                }
                PropertyKeyInner::Index(_) => {}
            }

            SetPropertyResult::Ok
        }
    }

    pub fn delete_property(&mut self, key: PropertyKey) -> Option<PropertyValue> {
        let index = self.find_key_index(key)? as usize;
        let value = self.property_value_at_index(index);

        fn delete_from_section<T>(ptr: *mut MaybeUninit<T>, index: usize, section_len: usize) {
            let section = unsafe { std::slice::from_raw_parts_mut(ptr, section_len) };
            section[index..].rotate_left(1);
        }

        let len = self.len() as usize;

        delete_from_section(self.data_mut().cast::<MaybeUninit<u32>>(), index, len);

        delete_from_section(
            self.values_start_mut()
                .cast::<MaybeUninit<InternalLinearPropertyVecValue>>(),
            index,
            len,
        );

        delete_from_section(
            self.descriptors_start_mut()
                .cast::<MaybeUninit<InternalLinearPropertyVecDescriptor>>(),
            index,
            len,
        );

        unsafe {
            (*self.0.as_ptr()).len -= 1;

            match key.inner() {
                PropertyKeyInner::String(_) => (*self.0.as_ptr()).string_key_count -= 1,
                PropertyKeyInner::Symbol(_) => (*self.0.as_ptr()).symbol_key_count -= 1,
                PropertyKeyInner::Index(_) => {}
            }
        }

        Some(value)
    }

    pub fn property_value_at_index(&self, index: usize) -> PropertyValue {
        assert!(index < self.len() as usize);

        let (values, descriptors) = self.values_descriptors();
        let value = unsafe { *values.get_unchecked(index) };
        let descriptor = unsafe { *descriptors.get_unchecked(index) };

        let mut norm_descriptor = PropertyDataDescriptor::empty();

        if descriptor.contains(InternalLinearPropertyVecDescriptor::CONFIGURABLE) {
            norm_descriptor |= PropertyDataDescriptor::CONFIGURABLE;
        }
        if descriptor.contains(InternalLinearPropertyVecDescriptor::ENUMERABLE) {
            norm_descriptor |= PropertyDataDescriptor::ENUMERABLE;
        }
        if descriptor.contains(InternalLinearPropertyVecDescriptor::WRITABLE) {
            norm_descriptor |= PropertyDataDescriptor::WRITABLE;
        }

        match descriptor.intersection(InternalLinearPropertyVecDescriptor::GET_SET) {
            InternalLinearPropertyVecDescriptor::GET => PropertyValue::new(
                PropertyValueKind::Trap {
                    get: Some(unsafe { value.trap.get.assume_init() }),
                    set: None,
                },
                norm_descriptor,
            ),
            InternalLinearPropertyVecDescriptor::SET => PropertyValue::new(
                PropertyValueKind::Trap {
                    get: None,
                    set: Some(unsafe { value.trap.set.assume_init() }),
                },
                norm_descriptor,
            ),
            InternalLinearPropertyVecDescriptor::GET_SET => PropertyValue::new(
                PropertyValueKind::Trap {
                    get: Some(unsafe { value.trap.get.assume_init() }),
                    set: Some(unsafe { value.trap.set.assume_init() }),
                },
                norm_descriptor,
            ),
            _ => PropertyValue::new(PropertyValueKind::Static(unsafe { value.static_ }), norm_descriptor),
        }
    }

    pub fn get_property(&self, key: PropertyKey) -> Option<PropertyValue> {
        match self.find_key_index(key) {
            Some(index) => Some(self.property_value_at_index(index as usize)),
            None => {
                cold_path();
                None
            }
        }
    }

    pub fn get_prototype(&self) -> PropertyValue {
        unsafe { (*self.0.as_ptr()).prototype }
    }

    pub fn get_prototype_mut(&mut self) -> &mut PropertyValue {
        unsafe { &mut (*self.0.as_ptr()).prototype }
    }

    fn find_key_index(&self, key: PropertyKey) -> Option<u32> {
        let (keys, off, search) = match key.inner() {
            PropertyKeyInner::String(js_string) => (self.string_keys(), 0, js_string.sym().raw()),
            PropertyKeyInner::Symbol(symbol) => (self.symbol_keys(), self.string_key_count(), symbol.sym().raw()),
            PropertyKeyInner::Index(sym) => (
                self.index_keys(),
                self.string_key_count() + self.symbol_key_count(),
                sym,
            ),
        };

        // For small N
        #[expect(clippy::manual_map)]
        unsafe fn check<const N: usize>(keys: &[u32], search: u32) -> Option<u32> {
            unsafe { std::hint::assert_unchecked(N <= keys.len()) };

            // TODO: make sure this gets inlined and unrolled
            match keys.iter().copied().position(|n| n == search) {
                Some(v) => Some(v as u32),
                None => None,
            }
        }

        let relative = match keys.len() {
            0 => None,
            1 => unsafe { check::<1>(keys, search) },
            2 => unsafe { check::<2>(keys, search) },
            3 => unsafe { check::<3>(keys, search) },
            4..8 => 'search: {
                // 4-8 properties: SSE2 search (compare 4 keys at once) if available, otherwise scalar loop

                let mut chunk_index = 0;

                #[cfg(target_arch = "x86_64")]
                let rem = if X86_FEATURES.sse2 {
                    // SAFETY: we checked that we have sse2 enabled
                    match unsafe { chunk_search4_sse2(keys, search, &mut chunk_index) } {
                        Ok(v) => break 'search Some(v),
                        Err(rem) => rem,
                    }
                } else {
                    keys
                };
                #[cfg(not(target_arch = "x86_64"))]
                let rem = keys;

                rem.iter()
                    .copied()
                    .position(|v| v == search)
                    .map(|v| v as u32 + chunk_index)
            }
            8.. => 'search: {
                // 8+ properties: either AVX search (8 keys at once), SSE2 (4 keys at once), or fall back to
                // a more portable (but slightly slower) autovectorized search (TODO)
                let cpufeatures = *X86_FEATURES;

                let mut chunk_index = 0;
                #[cfg(target_arch = "x86_64")]
                let rem = if cpufeatures.avx2 {
                    // SAFETY: we checked that we have avx enabled
                    match unsafe { chunk_search8_avx(keys, search, &mut chunk_index) } {
                        Ok(v) => break 'search Some(v),
                        Err(rem) => rem,
                    }
                } else if cpufeatures.sse2 {
                    // SAFETY: we checked that we have sse2 enabled
                    match unsafe { chunk_search4_sse2(keys, search, &mut chunk_index) } {
                        Ok(v) => break 'search Some(v),
                        Err(rem) => rem,
                    }
                } else {
                    keys
                };
                #[cfg(not(target_arch = "x86_64"))]
                let rem = keys;

                rem.iter()
                    .copied()
                    .position(|v| v == search)
                    .map(|v| v as u32 + chunk_index)
            }
        };

        relative.map(|v| v + off)
    }

    pub fn len(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).len }
    }

    pub fn capacity(&self) -> NonZero<u32> {
        unsafe { (*self.0.as_ptr()).cap }
    }

    /// Returns a pointer to the start of the DST data
    pub fn data(&self) -> *const () {
        unsafe { (&raw const (*self.0.as_ptr()).data).cast() }
    }

    pub fn data_mut(&mut self) -> *mut () {
        unsafe { (&raw mut (*self.0.as_ptr()).data).cast() }
    }

    /// Returns a pointer to the start of the string keys
    fn string_keys_start(&self) -> *const u32 {
        // Strings are always at the start of the data section
        self.data().cast()
    }

    /// Returns the number of string keys
    fn string_key_count(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).string_key_count }
    }

    /// Returns the number of symbol keys
    fn symbol_key_count(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).symbol_key_count }
    }

    /// Returns the number of index keys
    fn index_key_count(&self) -> u32 {
        unsafe {
            let ptr = self.0.as_ptr();
            (*ptr).len - ((*ptr).string_key_count + (*ptr).symbol_key_count)
        }
    }

    /// Returns a slice of string keys
    fn string_keys(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.string_keys_start(), self.string_key_count() as usize) }
    }

    /// Returns a pointer to the start of the symbol keys
    fn symbol_keys_start(&self) -> *const u32 {
        unsafe { self.string_keys_start().add(self.string_key_count() as usize) }
    }

    /// Returns a slice of symbol keys
    fn symbol_keys(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.symbol_keys_start(), self.symbol_key_count() as usize) }
    }

    /// The offset in counts of `size_of<u32>` at which symbol keys start.
    fn symbol_key_index_offset(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).string_key_count }
    }

    /// The offset in counts of `size_of<u32>` at which index keys start.
    fn index_key_index_offset(&self) -> u32 {
        unsafe { self.symbol_key_index_offset() + (*self.0.as_ptr()).symbol_key_count }
    }

    /// Returns a pointer to the start of the index keys
    fn index_keys_start(&self) -> *const u32 {
        unsafe { self.symbol_keys_start().add(self.symbol_key_count() as usize) }
    }

    /// Returns a slice of index keys
    fn index_keys(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.index_keys_start(), self.index_key_count() as usize) }
    }

    /// Returns a pointer to the start of the values section
    pub fn values_start(&self) -> *const InternalLinearPropertyVecValue {
        let cap = self.capacity().get();
        let base = unsafe {
            self.data()
                .cast::<u32>()
                .add(cap as usize)
                .cast::<InternalLinearPropertyVecValue>()
        };
        unsafe { align_ptr(base) }
    }

    /// Returns a mutable pointer to the start of the values section
    pub fn values_start_mut(&mut self) -> *mut InternalLinearPropertyVecValue {
        let cap = self.capacity().get();
        let base = unsafe {
            self.data_mut()
                .cast::<u32>()
                .add(cap as usize)
                .cast::<InternalLinearPropertyVecValue>()
        };
        unsafe { align_ptr_mut(base) }
    }

    /// Returns a mutable slice of values and descriptors.
    pub fn values_descriptors_mut(
        &mut self,
    ) -> (
        &mut [InternalLinearPropertyVecValue],
        &mut [InternalLinearPropertyVecDescriptor],
    ) {
        let values = self.values_start_mut();
        let descriptors = unsafe {
            align_ptr_mut(
                values
                    .add(self.capacity().get() as usize)
                    .cast::<InternalLinearPropertyVecDescriptor>(),
            )
        };
        let len = self.len() as usize;
        (unsafe { std::slice::from_raw_parts_mut(values, len) }, unsafe {
            std::slice::from_raw_parts_mut(descriptors, len)
        })
    }

    pub fn values_descriptors(
        &self,
    ) -> (
        &[InternalLinearPropertyVecValue],
        &[InternalLinearPropertyVecDescriptor],
    ) {
        let values = self.values_start();
        let descriptors = unsafe {
            align_ptr(
                values
                    .add(self.capacity().get() as usize)
                    .cast::<InternalLinearPropertyVecDescriptor>(),
            )
        };
        let len = self.len() as usize;
        (unsafe { std::slice::from_raw_parts(values, len) }, unsafe {
            std::slice::from_raw_parts(descriptors, len)
        })
    }

    pub fn descriptors_start_mut(&mut self) -> *mut InternalLinearPropertyVecDescriptor {
        let cap = self.capacity().get() as usize;
        let base = unsafe {
            self.values_start_mut()
                .add(cap)
                .cast::<InternalLinearPropertyVecDescriptor>()
        };
        unsafe { align_ptr_mut(base) }
    }

    fn raw_keys(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.data().cast::<u32>(), self.len() as usize) }
    }
}

impl Drop for PropertyVec {
    fn drop(&mut self) {
        // SAFETY: we always allocate with `Self::layout(cap)`, so deallocating with that is safe
        let layout = Self::layout(self.capacity());
        unsafe { std::alloc::dealloc(self.0.as_ptr().cast(), layout) };
    }
}

/// SAFETY:
/// * sse2 must be available
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn chunk_search4_sse2<'a>(slice: &'a [u32], search: u32, chunk_index: &mut u32) -> Result<u32, &'a [u32]> {
    use std::arch::x86_64::*;

    debug_assert!(X86_FEATURES.sse2);
    let search = _mm_set1_epi32(search.cast_signed());

    let mut chunks = slice.chunks_exact(4);
    for chunk in chunks.by_ref() {
        if let Some(value) = unsafe { search4_sse2(chunk, search) } {
            return Ok(*chunk_index + value);
        }
        *chunk_index += 4;
    }
    Err(chunks.remainder())
}

/// SAFETY:
/// * sse2 must be available
/// * slice must contain at least 4 elements
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn search4_sse2(slice: &[u32], search: std::arch::x86_64::__m128i) -> Option<u32> {
    use std::arch::x86_64::*;

    debug_assert!(X86_FEATURES.sse2);
    unsafe { std::hint::assert_unchecked(slice.len() >= 4) };

    let vec = unsafe { _mm_loadu_si128(slice.as_ptr().cast()) };
    let mask = _mm_cmpeq_epi32(vec, search);
    let mask = _mm_movemask_epi8(mask);

    if mask != 0 {
        Some(mask.trailing_zeros() >> 2)
    } else {
        None
    }
}

/// SAFETY:
/// * avx2 must be available
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn chunk_search8_avx<'a>(slice: &'a [u32], search: u32, chunk_index: &mut u32) -> Result<u32, &'a [u32]> {
    use std::arch::x86_64::*;

    debug_assert!(X86_FEATURES.avx2);
    let search = _mm256_set1_epi32(search.cast_signed());

    let mut chunks = slice.chunks_exact(8);
    for chunk in chunks.by_ref() {
        if let Some(value) = unsafe { search8_avx2(chunk, search) } {
            return Ok(*chunk_index + value);
        }
        *chunk_index += 8;
    }
    Err(chunks.remainder())
}

/// SAFETY:
/// * avx2 must be available
/// * slice must contain at least 8 elements
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn search8_avx2(slice: &[u32], search: std::arch::x86_64::__m256i) -> Option<u32> {
    use std::arch::x86_64::*;

    debug_assert!(X86_FEATURES.avx2);
    unsafe { std::hint::assert_unchecked(slice.len() >= 8) };

    let vec = unsafe { _mm256_loadu_si256(slice.as_ptr().cast()) };
    let mask = _mm256_cmpeq_epi32(vec, search);
    let mask = _mm256_movemask_epi8(mask);

    if mask != 0 {
        Some(mask.trailing_zeros() >> 2)
    } else {
        None
    }
}

#[repr(align(4))]
struct Aligned4Zst;

/// `data` format:
///
/// | Keys                                         | Values                                             | Descriptor bits (V=Value,T=Trap)  |
/// |----------------------------------------------|----------------------------------------------------|-----------------------------------|
/// | K0,K1,K2,K3,K4,...,Kn   <... spare capacity> | (0, null), (getter, setter) <... spare capacity>   | CWEV,CWET <... spare capacity>    |
///
/// Storing the interned u32 keys all next to each other optimizes cache coherency and allows for more optimized SSE2/AVX2 search.
///
/// The order of keys are (from most likely to least likely):
/// 1.) String keys (`PropertyKeyInner::String`)
/// 2.) Symbol keys (`PropertyKeyInner::Symbol`)
/// 3.) Index keys (`PropertyKeyInner::Index`)
#[repr(C)]
struct PropertyVecAllocation {
    /// The number of properties (or IOW, length of each section). That is, if len is 3, then there are 3 keys, 3 values and 3 descriptor bit entries.
    len: u32,
    /// How many properties can we store in this allocation without needing to allocate. This is always `>=` than the length.
    cap: NonZero<u32>,
    string_key_count: u32,
    symbol_key_count: u32,
    // TODO: it really just needs to be a Value
    prototype: PropertyValue,
    data: [Aligned4Zst; 0],
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::Vm;
    use crate::value::Value;
    use crate::value::object::{Object, PropertyValue};
    use crate::value::primitive::Symbol;
    use crate::value::propertykey::ToPropertyKey;
    use crate::value::string::JsString;

    use super::OrdObject;

    #[test]
    fn bench() {
        if true {
            return;
        }
        let mut vm = Vm::new(Default::default());
        let sc = &mut vm.scope();
        let obj = sc.register(OrdObject::null());
        let props = std::env::var("PROP_COUNT").unwrap().parse::<usize>().unwrap();

        for i in 0..props {
            let k1 = sc.intern(format!("k{i}"));
            let symk = Symbol::new(k1.into()).to_key(sc);
            let symv = PropertyValue::static_default(Value::number((i + 10) as f64));
            let strk = JsString::from_sym(k1).to_key(sc);
            let strv = PropertyValue::static_default(Value::number((i + 100) as f64));
            let numk = i.to_key(sc);
            let numv = PropertyValue::static_default(Value::number((i + 1000) as f64));

            obj.set_property(symk, symv, sc).unwrap();
            obj.set_property(strk, strv, sc).unwrap();
            obj.set_property(numk, numv, sc).unwrap();
        }

        let syms = (0..props)
            .map(|v| JsString::from_sym(sc.intern(format!("k{v}").as_str())).to_key(sc))
            .collect::<Vec<_>>();
        let count = std::env::var("ITER_COUNT").unwrap().parse::<usize>().unwrap();

        for _ in 0..32 {
            let sym = std::hint::black_box(syms[0]);
            let i = Instant::now();
            for _ in 0..count {
                obj.get_property(sym, sc).unwrap();
            }
            let elapsed = i.elapsed();
            dbg!(elapsed / (count * syms.len()) as u32);
        }
    }

    #[test]
    fn basic_ops() {
        let mut vm = Vm::new(Default::default());
        let sc = &mut vm.scope();
        let obj = OrdObject::null();

        for i in 0..100 {
            let k1 = sc.intern(format!("k{i}"));
            let symk = Symbol::new(k1.into()).to_key(sc);
            let symv = PropertyValue::static_default(Value::number((i + 10) as f64));
            let strk = JsString::from_sym(k1).to_key(sc);
            let strv = PropertyValue::static_default(Value::number((i + 100) as f64));
            let numk = i.to_key(sc);
            let numv = PropertyValue::static_default(Value::number((i + 1000) as f64));

            // Test property setting
            obj.set_property(symk, symv, sc).unwrap();
            obj.set_property(strk, strv, sc).unwrap();
            obj.set_property(numk, numv, sc).unwrap();

            assert_eq!(obj.get_own_property_descriptor(symk, sc).unwrap().unwrap(), symv);
            assert_eq!(obj.get_own_property_descriptor(strk, sc).unwrap().unwrap(), strv);
            assert_eq!(obj.get_own_property_descriptor(numk, sc).unwrap().unwrap(), numv);

            // Test deleting
            obj.delete_property(strk, sc).unwrap();

            assert_eq!(obj.get_own_property_descriptor(strk, sc).unwrap(), None);
            assert_eq!(obj.get_own_property_descriptor(symk, sc).unwrap().unwrap(), symv);
            assert_eq!(obj.get_own_property_descriptor(numk, sc).unwrap().unwrap(), numv);
            if i % 2 == 0 {
                obj.delete_property(symk, sc).unwrap();
                assert_eq!(obj.get_own_property_descriptor(strk, sc).unwrap(), None);
                assert_eq!(obj.get_own_property_descriptor(symk, sc).unwrap(), None);
                assert_eq!(obj.get_own_property_descriptor(numk, sc).unwrap().unwrap(), numv);
            }
            if i % 8 == 0 {
                obj.delete_property(numk, sc).unwrap();
                assert_eq!(obj.get_own_property_descriptor(strk, sc).unwrap(), None);
                assert_eq!(obj.get_own_property_descriptor(symk, sc).unwrap(), None);
                assert_eq!(obj.get_own_property_descriptor(numk, sc).unwrap(), None);
            }
        }
    }
}
