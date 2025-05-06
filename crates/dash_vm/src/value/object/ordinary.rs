use std::alloc::Layout;
use std::cell::RefCell;
use std::iter;
use std::mem::MaybeUninit;
use std::num::NonZero;
use std::ptr::{self, NonNull};
use std::sync::LazyLock;

use dash_middle::interner::{self};

use crate::frame::This;
use crate::gc::ObjectId;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::localscope::LocalScope;
use crate::value::function::args::CallArgs;
use crate::value::primitive::Symbol;
use crate::value::propertykey::{PropertyKey, PropertyKeyInner, ToPropertyKey};
use crate::value::string::JsString;
use crate::value::{Root, Unpack, Unrooted, Value, ValueKind};
use crate::{Vm, extract, throw};

use super::{Object, PropertyDataDescriptor, PropertyValue, PropertyValueKind};

#[derive(Copy, Clone)]
struct X86Features {
    sse2: bool,
    avx: bool,
}

static X86_FEATURES: LazyLock<X86Features> = LazyLock::new(|| X86Features {
    sse2: is_x86_feature_detected!("sse2"),
    avx: is_x86_feature_detected!("avx"),
});

#[derive(Debug)]
enum InnerOrdObject {
    // TODO: prototype should just be Option<ObjectId>..?
    Cow { prototype: PropertyValue },
    Linear(PropertyVec),
}

/// An **ordinary** object (object with default behavior for the internal methods).
#[derive(Debug)]
pub struct OrdObject(RefCell<InnerOrdObject>);

unsafe impl Trace for OrdObject {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match &*self.0.borrow() {
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
                    if descriptor.is_trap() {
                        let (get, set) = unsafe { value.trap };
                        get.trace(cx);
                        set.trace(cx);
                    } else {
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
        Self(RefCell::new(InnerOrdObject::Cow {
            prototype: PropertyValue::static_default(Value::object(vm.statics.object_prototype)),
        }))
    }

    pub fn with_prototype(prototype: ObjectId) -> Self {
        Self(RefCell::new(InnerOrdObject::Cow {
            prototype: PropertyValue::static_default(prototype.into()),
        }))
    }

    pub fn with_prototype_and_ctor(prototype: ObjectId, ctor: ObjectId) -> Self {
        let mut pvec = PropertyVec::new(PropertyValue::static_default(prototype.into()));
        pvec.set_property(PropertyKey::CONSTRUCTOR, PropertyValue::static_default(ctor.into()));
        Self(RefCell::new(InnerOrdObject::Linear(pvec)))
    }

    pub fn null() -> Self {
        Self(RefCell::new(InnerOrdObject::Cow {
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
}

impl Object for OrdObject {
    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        _: &mut LocalScope<'_>,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        // TODO: stop handling __proto__ here and do that in the handler instead?

        match *self.0.borrow() {
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

    fn set_property(&self, key: PropertyKey, value: PropertyValue, sc: &mut LocalScope<'_>) -> Result<(), Value> {
        fn assign_prototype(
            value: PropertyValue,
            prototype: &mut PropertyValue,
            sc: &mut LocalScope<'_>,
        ) -> Result<(), Value> {
            match prototype.kind() {
                PropertyValueKind::Static(..) => {
                    *prototype = value;
                }
                PropertyValueKind::Trap { set, .. } => {
                    if let Some(set) = set {
                        if let PropertyValueKind::Static(value) = value.kind() {
                            // FIXME: set a proper `this` binding
                            // TODO: this can panic if we re-enter set_property function due to refcell guard
                            match set.apply(This::Default, [*value].into(), sc) {
                                Ok(_) => return Ok(()),
                                Err(err) => return Err(err.root(sc)),
                            }
                        }
                    }

                    *prototype = value;
                }
            }

            Ok(())
        }

        let mut guard = self.0.borrow_mut();

        match &mut *guard {
            InnerOrdObject::Cow { prototype } => {
                if key == PropertyKey::PROTO {
                    assign_prototype(value, prototype, sc)
                } else {
                    let prototype = *prototype;
                    *guard = InnerOrdObject::Linear(PropertyVec::new(prototype));
                    drop(guard);
                    self.set_property(key, value, sc)
                }
            }
            InnerOrdObject::Linear(property_vec) => {
                if key == PropertyKey::PROTO {
                    assign_prototype(value, property_vec.get_prototype_mut(), sc)
                } else {
                    match property_vec.set_property(key, value) {
                        SetPropertyResult::Ok => Ok(()),
                        // TODO: throw in strict mode?
                        SetPropertyResult::NotWritable => Ok(()),
                        SetPropertyResult::InvokeSetter(alloc_id) => {
                            drop(guard);

                            if let PropertyValueKind::Static(value) = value.kind() {
                                match alloc_id.apply(This::Default, [*value].into(), sc) {
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
        match &mut *self.0.borrow_mut() {
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
        match &mut *self.0.borrow_mut() {
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
        match *self.0.borrow() {
            // FIXME: proper `this` binding to the object
            InnerOrdObject::Cow { prototype } => prototype.get_or_apply(sc, This::Default).root(sc),
            InnerOrdObject::Linear(ref property_vec) => {
                property_vec.get_prototype().get_or_apply(sc, This::Default).root(sc)
            }
        }
    }

    fn apply(&self, _: ObjectId, _: This, _: CallArgs, scope: &mut LocalScope<'_>) -> Result<Unrooted, Unrooted> {
        throw!(scope, Error, "Attempted to call non-function object")
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        match *self.0.borrow() {
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
    #[derive(Debug, Copy, Clone)]
    struct InternalLinearPropertyVecDescriptor: u8 {
        const CONFIGURABLE = 1 << 0;
        const ENUMERABLE = 1 << 1;
        const WRITABLE = 1 << 2;
        const TRAP = 1 << 3;
    }
}

impl InternalLinearPropertyVecDescriptor {
    fn is_trap(self) -> bool {
        self.contains(Self::TRAP)
    }
}

// discriminant indicated by `InternalLinearPropertyVecDescriptor`
#[derive(Copy, Clone)]
union InternalLinearPropertyVecValue {
    // TODO: we really need ObjectId to have a niche
    trap: (Option<ObjectId>, Option<ObjectId>),
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
            descriptor |= InternalLinearPropertyVecDescriptor::TRAP;
            InternalLinearPropertyVecValue { trap: (get, set) }
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

            if descriptor.is_trap() {
                let (_, set) = unsafe { value.trap };
                if let Some(set) = set {
                    return SetPropertyResult::InvokeSetter(set);
                }
            }

            *value = val;
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
                PropertyKeyInner::Index(idx) => (self.index_key_index_offset() + self.index_key_count(), idx), // TODO: is -1 really correct?
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
                    descriptor |= InternalLinearPropertyVecDescriptor::TRAP;
                    InternalLinearPropertyVecValue { trap: (get, set) }
                }
                PropertyValueKind::Static(value) => InternalLinearPropertyVecValue { static_: value },
            };

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
        let index = self.find_key_index(key)?;
        let value = self.property_value_at_index(index as usize);

        // Goal: Swap the last element with the one we want to delete, then shift everything after the last element 1 to the left.

        let last_index = match key.inner() {
            PropertyKeyInner::String(..) => self.symbol_key_index_offset() - 1,
            PropertyKeyInner::Symbol(..) => self.index_key_index_offset() - 1,
            PropertyKeyInner::Index(_) => self.index_key_index_offset() + self.index_key_count() - 1,
        };

        fn delete_from_section<T>(ptr: *mut MaybeUninit<T>, index: usize, last_index: usize, section_len: usize) {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, section_len) };
            slice.swap(index, last_index);
            slice[last_index..].rotate_left(1);
        }

        let len = self.len() as usize;

        delete_from_section(
            self.data_mut().cast::<MaybeUninit<u32>>(),
            index as usize,
            last_index as usize,
            len,
        );

        delete_from_section(
            self.values_start_mut()
                .cast::<MaybeUninit<InternalLinearPropertyVecValue>>(),
            index as usize,
            len - 1,
            len,
        );

        delete_from_section(
            self.descriptors_start_mut()
                .cast::<MaybeUninit<InternalLinearPropertyVecDescriptor>>(),
            index as usize,
            len - 1,
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

    fn property_value_at_index(&self, index: usize) -> PropertyValue {
        assert!(index < self.len() as usize);
        let (values, descriptors) = self.values_descriptors();
        let value = unsafe { *values.get_unchecked(index) };
        let descriptor = unsafe { *descriptors.get_unchecked(index) };
        // let value = self.values()[index];
        // let descriptor = self.descriptors()[index];
        let mut norm_descriptor = PropertyDataDescriptor::empty();

        // TODO: make sure this compiles to good asm
        if descriptor.contains(InternalLinearPropertyVecDescriptor::CONFIGURABLE) {
            norm_descriptor |= PropertyDataDescriptor::CONFIGURABLE;
        }
        if descriptor.contains(InternalLinearPropertyVecDescriptor::ENUMERABLE) {
            norm_descriptor |= PropertyDataDescriptor::ENUMERABLE;
        }
        if descriptor.contains(InternalLinearPropertyVecDescriptor::WRITABLE) {
            norm_descriptor |= PropertyDataDescriptor::WRITABLE;
        }

        if descriptor.is_trap() {
            let (get, set) = unsafe { value.trap };

            PropertyValue::new(PropertyValueKind::Trap { get, set }, norm_descriptor)
        } else {
            PropertyValue::new(PropertyValueKind::Static(unsafe { value.static_ }), norm_descriptor)
        }
    }

    pub fn get_property(&self, key: PropertyKey) -> Option<PropertyValue> {
        let index = self.find_key_index(key)? as usize;
        Some(self.property_value_at_index(index))
    }

    pub fn get_prototype(&self) -> PropertyValue {
        unsafe { (*self.0.as_ptr()).prototype }
    }

    pub fn get_prototype_mut(&mut self) -> &mut PropertyValue {
        unsafe { &mut (*self.0.as_ptr()).prototype }
    }

    fn find_key_index(&self, key: PropertyKey) -> Option<u32> {
        let (keys, search) = match key.inner() {
            PropertyKeyInner::String(js_string) => (self.string_keys(), js_string.sym().raw()),
            PropertyKeyInner::Symbol(symbol) => (self.symbol_keys(), symbol.sym().raw()),
            PropertyKeyInner::Index(sym) => (self.index_keys(), sym),
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

        match keys.len() {
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
                let rem = if cpufeatures.avx {
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
                    // TODO: portable autovectorized loops
                    keys
                };
                #[cfg(not(target_arch = "x86_64"))]
                let rem = keys;

                rem.iter()
                    .copied()
                    .position(|v| v == search)
                    .map(|v| v as u32 + chunk_index)
            }
        }
    }

    pub fn len(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).len }
    }

    pub fn capacity(&self) -> NonZero<u32> {
        unsafe { (*self.0.as_ptr()).cap }
    }

    pub fn data(&self) -> *const () {
        unsafe { (&raw const (*self.0.as_ptr()).data).cast() }
    }

    pub fn data_mut(&mut self) -> *mut () {
        unsafe { (&raw mut (*self.0.as_ptr()).data).cast() }
    }

    fn string_keys_start(&self) -> *const u32 {
        self.data().cast()
    }

    fn string_key_count(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).string_key_count }
    }

    fn symbol_key_count(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).symbol_key_count }
    }

    fn index_key_count(&self) -> u32 {
        unsafe {
            let ptr = self.0.as_ptr();
            (*ptr).len - ((*ptr).string_key_count + (*ptr).symbol_key_count)
        }
    }

    fn string_keys(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.string_keys_start(), self.string_key_count() as usize) }
    }

    fn symbol_keys_start(&self) -> *const u32 {
        unsafe { self.string_keys_start().add(self.string_key_count() as usize) }
    }

    fn symbol_keys(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.symbol_keys_start(), self.symbol_key_count() as usize) }
    }

    /// The offset in counts of `size_of<u32>` at which string keys start.
    fn string_key_index_offset(&self) -> u32 {
        0
    }

    /// The offset in counts of `size_of<u32>` at which symbol keys start.
    fn symbol_key_index_offset(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).string_key_count }
    }

    /// The offset in counts of `size_of<u32>` at which index keys start.
    fn index_key_index_offset(&self) -> u32 {
        unsafe { self.symbol_key_index_offset() + (*self.0.as_ptr()).symbol_key_count }
    }

    fn index_keys_start(&self) -> *const u32 {
        unsafe { self.symbol_keys_start().add(self.symbol_key_count() as usize) }
    }

    fn index_keys(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.index_keys_start(), self.index_key_count() as usize) }
    }

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
                    .add(self.len() as usize)
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
                    .add(self.len() as usize)
                    .cast::<InternalLinearPropertyVecDescriptor>(),
            )
        };
        let len = self.len() as usize;
        (unsafe { std::slice::from_raw_parts(values, len) }, unsafe {
            std::slice::from_raw_parts(descriptors, len)
        })
    }

    pub fn values(&self) -> &[InternalLinearPropertyVecValue] {
        unsafe { std::slice::from_raw_parts(self.values_start(), self.len() as usize) }
    }

    pub fn descriptors_start(&self) -> *const InternalLinearPropertyVecDescriptor {
        let cap = self.capacity().get() as usize;
        let base = unsafe {
            self.values_start()
                .add(cap)
                .cast::<InternalLinearPropertyVecDescriptor>()
        };
        unsafe { align_ptr(base) }
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

    pub fn descriptors(&self) -> &[InternalLinearPropertyVecDescriptor] {
        let cap = self.capacity().get() as usize;
        let len = self.len() as usize;
        let base = unsafe {
            self.values_start()
                .add(cap)
                .cast::<InternalLinearPropertyVecDescriptor>()
        };
        let ptr = unsafe { align_ptr(base) };
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    fn raw_keys(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.data().cast::<u32>(), self.len() as usize) }
    }

    fn raw_keys_mut(&mut self) -> &mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(self.data_mut().cast::<u32>(), self.len() as usize) }
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
unsafe fn chunk_search4_sse2<'a>(slice: &'a [u32], search: u32, chunk_index: &mut u32) -> Result<u32, &'a [u32]> {
    use std::arch::x86_64::*;

    debug_assert!(X86_FEATURES.sse2);
    let search = unsafe { _mm_set1_epi32(search.cast_signed()) };

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
#[cfg(target_arch = "x86_64")]
unsafe fn search4_sse2(slice: &[u32], search: std::arch::x86_64::__m128i) -> Option<u32> {
    use std::arch::x86_64::*;

    debug_assert!(X86_FEATURES.sse2);
    unsafe { std::hint::assert_unchecked(slice.len() >= 4) };

    // // SAFETY: this is a `&[u32]`, so it's properly aligned, and the caller ensures that len >= 4
    let vec = unsafe { _mm_loadu_si128(slice.as_ptr().cast()) };
    let mask = unsafe { _mm_cmpeq_epi32(vec, search) };
    let mask = unsafe { _mm_movemask_epi8(mask) };

    if mask != 0 {
        Some(mask.trailing_zeros() >> 2)
    } else {
        None
    }
}

/// SAFETY:
/// * avx must be available
#[cfg(target_arch = "x86_64")]
unsafe fn chunk_search8_avx<'a>(slice: &'a [u32], search: u32, chunk_index: &mut u32) -> Result<u32, &'a [u32]> {
    use std::arch::x86_64::*;

    debug_assert!(X86_FEATURES.avx);
    let search = unsafe { _mm256_set1_epi32(search.cast_signed()) };

    let mut chunks = slice.chunks_exact(8);
    for chunk in chunks.by_ref() {
        if let Some(value) = unsafe { search8_avx(chunk, search) } {
            return Ok(*chunk_index + value);
        }
        *chunk_index += 8;
    }
    Err(chunks.remainder())
}

/// SAFETY:
/// * avx must be available
#[cfg(target_arch = "x86_64")]
unsafe fn search8_avx(slice: &[u32], search: std::arch::x86_64::__m256i) -> Option<u32> {
    use std::arch::x86_64::*;

    debug_assert!(X86_FEATURES.avx);
    unsafe { std::hint::assert_unchecked(slice.len() >= 8) };

    // SAFETY: this is a `&[u32]`, so it's properly aligned, and the caller ensures that len >= 8
    let vec = unsafe { _mm256_loadu_si256(slice.as_ptr().cast()) };
    let mask = unsafe { _mm256_cmpeq_epi32(vec, search) };
    let mask = unsafe { _mm256_movemask_epi8(mask) };

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
/// Storing the interned u32 keys all next to each other optimizes cache coherency and allows for more optimized SSE2/AVX search.
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
    prototype: PropertyValue,
    data: [Aligned4Zst; 0],
}
