use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::mem;

use dash_log::debug;
use dash_proc_macro::Trace;
use table::ArrayTable;

use crate::frame::This;
use crate::gc::ObjectId;
use crate::gc::trace::Trace;
use crate::localscope::LocalScope;
use crate::value::object::PropertyDataDescriptor;
use crate::{Vm, delegate, extract, throw};
use dash_middle::interner::sym;

use super::object::{NamedObject, Object, PropertyKey, PropertyValue, PropertyValueKind};
use super::ops::conversions::ValueConversion;
use super::primitive::array_like_keys;
use super::root_ext::RootErrExt;
use super::{Root, Unpack, Unrooted, Value};

pub mod table;

pub const MAX_LENGTH: u32 = 4294967295;
pub const MAX_INDEX: u32 = MAX_LENGTH - 1;

pub fn require_valid_array_length(scope: &mut LocalScope<'_>, len: usize) -> Result<(), Value> {
    if len > MAX_LENGTH as usize {
        throw!(scope, RangeError, "Invalid array length");
    }
    Ok(())
}

#[derive(Debug)]
pub enum ArrayInner {
    // TODO: store Value, also support holes
    // TODO: move away from `Vec`? we don't need a `usize` for the length as the max size fits in a u32
    Dense(Vec<PropertyValue>),
    Table(ArrayTable),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MaybeHoley<T> {
    Some(T),
    Hole,
}

impl ArrayInner {
    /// Computes the length (the highest index at which an element is stored + 1)
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32 {
        match self {
            ArrayInner::Dense(v) => v.len() as u32,
            ArrayInner::Table(v) => v.len(),
        }
    }

    pub fn get(&self, at: u32) -> Option<MaybeHoley<PropertyValue>> {
        match self {
            ArrayInner::Dense(v) => v.get(at as usize).cloned().map(MaybeHoley::Some),
            ArrayInner::Table(v) => v.get(at),
        }
    }

    fn transition_to_table(&mut self) {
        if let ArrayInner::Dense(v) = self {
            let len = v.len();
            let table = ArrayTable::from_iter(mem::take(v), len as u32);
            *self = ArrayInner::Table(table);
        }
    }

    fn transition_to_dense_if_no_holes(&mut self) {
        if let ArrayInner::Table(table) = self {
            if !table.has_holes() {
                *self = Self::Dense(table.take_into_sorted_array());
            }
        }
    }

    /// Resizes this array, potentially switching to a holey kind.
    pub fn resize(&mut self, new_length: u32) {
        match self {
            ArrayInner::Dense(v) => {
                let len = v.len();
                if new_length as usize <= len {
                    v.truncate(new_length as usize);
                } else {
                    debug!("out of bounds resize, convert to holey array");

                    let table = ArrayTable::from_iter(mem::take(v), new_length);
                    *self = ArrayInner::Table(table);
                }
            }
            ArrayInner::Table(v) => v.resize(new_length),
        }
    }

    pub fn set(&mut self, at: u32, value: PropertyValue) {
        match self {
            ArrayInner::Dense(v) => {
                match (at as usize).cmp(&v.len()) {
                    Ordering::Less => v[at as usize] = value,
                    Ordering::Equal => v.push(value),
                    Ordering::Greater => {
                        // resize us, causing self to have a hole and do the set logic below
                        self.resize(at + 1);
                        self.set(at, value);
                        debug_assert!(matches!(self, Self::Table(_)));
                    }
                }
            }
            ArrayInner::Table(v) => {
                v.set(at, value);
                self.transition_to_dense_if_no_holes();
            }
        }
    }

    pub fn push(&mut self, value: PropertyValue) {
        match self {
            ArrayInner::Dense(v) => v.push(value),
            ArrayInner::Table(v) => v.push(value),
        }
    }

    pub fn delete(&mut self, at: u32) -> Option<MaybeHoley<PropertyValue>> {
        match self {
            ArrayInner::Dense(v) => {
                if (at as usize) < v.len() {
                    // Deleting an element in the middle means there will be a hole, so transition to array table
                    self.transition_to_table();
                    self.delete(at)
                } else {
                    None
                }
            }
            ArrayInner::Table(v) => v.delete_make_hole(at),
        }
    }
}

unsafe impl Trace for ArrayInner {
    fn trace(&self, cx: &mut crate::gc::trace::TraceCtxt<'_>) {
        match self {
            ArrayInner::Dense(v) => v.trace(cx),
            ArrayInner::Table(v) => v.trace(cx),
        }
    }
}

#[derive(Debug, Trace)]
pub struct Array {
    pub items: RefCell<ArrayInner>,
    obj: NamedObject,
}

fn get_named_object(vm: &Vm) -> NamedObject {
    NamedObject::with_prototype_and_constructor(vm.statics.array_prototype, vm.statics.array_ctor)
}

impl Array {
    pub fn new(vm: &Vm) -> Self {
        Self::with_obj(get_named_object(vm))
    }

    /// Creates a non-holey array from a vec of values
    pub fn from_vec(vm: &Vm, items: Vec<PropertyValue>) -> Self {
        Self {
            items: RefCell::new(ArrayInner::Dense(items)),
            obj: get_named_object(vm),
        }
    }

    pub fn from_table(vm: &Vm, table: ArrayTable) -> Self {
        Self {
            items: RefCell::new(ArrayInner::Table(table)),
            obj: get_named_object(vm),
        }
    }

    /// Creates a holey array with a given length
    pub fn with_hole(vm: &Vm, len: usize) -> Self {
        Self {
            items: RefCell::new(ArrayInner::Table(ArrayTable::with_len(len as u32))),
            obj: get_named_object(vm),
        }
    }

    /// Tries to convert this holey array into a non-holey array
    pub fn try_convert_to_non_holey(&self) {
        self.items.borrow_mut().transition_to_dense_if_no_holes();
    }

    /// Converts this potentially-holey array into a non-holey array, assuming that it succeeds.
    /// In other words, this assumes that there aren't any holes and change the kind to be non-holey.
    ///
    /// This can be useful to call after an operation that is guaranteed to remove any holes (e.g. filling an array)
    pub fn force_convert_to_non_holey(&self) {
        self.try_convert_to_non_holey();
        assert!(matches!(*self.items.borrow(), ArrayInner::Dense(_)));
    }

    pub fn with_obj(obj: NamedObject) -> Self {
        Self {
            items: RefCell::new(ArrayInner::Dense(Vec::new())),
            obj,
        }
    }
}

impl Object for Array {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        let items = self.items.borrow();

        if let PropertyKey::String(key) = &key {
            if key.sym() == sym::length {
                return Ok(Some(PropertyValue {
                    kind: PropertyValueKind::Static(Value::number(items.len() as f64)),
                    descriptor: PropertyDataDescriptor::WRITABLE,
                }));
            }

            if let Ok(index) = key.res(sc).parse::<u32>() {
                if index < MAX_LENGTH {
                    if let Some(element) = items.get(index) {
                        match element {
                            MaybeHoley::Some(v) => return Ok(Some(v)),
                            MaybeHoley::Hole => return Ok(None),
                        }
                    }
                }
            }
        }

        self.obj.get_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        if let PropertyKey::String(key) = &key {
            if key.sym() == sym::length {
                // TODO: this shouldnt be undefined
                let value = value.kind().get_or_apply(sc, This::Default).root(sc)?;
                if let Ok(new_len) = u32::try_from(value.to_number(sc)? as usize) {
                    self.items.borrow_mut().resize(new_len);
                    return Ok(());
                }

                throw!(sc, RangeError, "Invalid array length")
            }

            if let Ok(index) = key.res(sc).parse::<u32>() {
                if index < MAX_LENGTH {
                    self.items.borrow_mut().set(index, value);

                    return Ok(());
                }
            }
        }

        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
        if let PropertyKey::String(key) = &key {
            if key.sym() == sym::length {
                return Ok(Unrooted::new(Value::undefined()));
            }

            if let Ok(index) = key.res(sc).parse::<u32>() {
                if index < MAX_LENGTH {
                    let mut items = self.items.borrow_mut();
                    match items.delete(index) {
                        Some(MaybeHoley::Some(value)) => {
                            return value.get_or_apply(sc, This::Default).root_err(sc);
                        }
                        Some(MaybeHoley::Hole) | None => return Ok(Value::undefined().into()),
                    }
                }
            }
        }

        self.obj.delete_property(sc, key)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: This,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(scope, callee, this, args)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        let items = self.items.borrow();
        // TODO: this should not include holey indices
        Ok(array_like_keys(sc, items.len() as usize).collect())
    }

    extract!(self);
}

#[derive(Debug, Trace)]
pub struct ArrayIterator {
    index: Cell<usize>,
    length: usize,
    value: Value,
    obj: NamedObject,
}

impl Object for ArrayIterator {
    delegate!(
        obj,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        own_keys
    );

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: This,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(scope, callee, this, args)
    }

    extract!(self);
}

impl ArrayIterator {
    pub fn new(sc: &mut LocalScope, value: Value) -> Result<Self, Value> {
        let length = value.length_of_array_like(sc)?;

        Ok(ArrayIterator {
            index: Cell::new(0),
            length,
            value,
            obj: NamedObject::with_prototype_and_constructor(
                sc.statics.array_iterator_prototype,
                sc.statics.object_ctor,
            ),
        })
    }

    pub fn empty() -> Self {
        Self {
            index: Cell::new(0),
            length: 0,
            value: Value::null(),
            obj: NamedObject::null(),
        }
    }

    pub fn next(&self, sc: &mut LocalScope) -> Result<Option<Unrooted>, Unrooted> {
        let index = self.index.get();

        if index < self.length {
            self.index.set(index + 1);
            let index = sc.intern_usize(index);
            self.value.get_property(sc, index.into()).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

/// Equivalent to calling get_property, but specialized for arrays
pub fn spec_array_get_property(scope: &mut LocalScope<'_>, target: &Value, index: usize) -> Result<Unrooted, Unrooted> {
    if let Ok(index) = u32::try_from(index) {
        if index < MAX_LENGTH {
            if let Some(arr) = target.unpack().downcast_ref::<Array>(scope) {
                let inner = arr.items.borrow();
                return match inner.get(index) {
                    Some(MaybeHoley::Some(value)) => value.get_or_apply(scope, This::Default),
                    Some(MaybeHoley::Hole) | None => Ok(Value::undefined().into()),
                };
            }
        }
    }

    let index = scope.intern_usize(index);
    match target.get_property(scope, index.into()) {
        Ok(v) => Ok(v),
        Err(v) => Ok(v),
    }
}

/// Equivalent to calling set_property, but specialized for arrays
pub fn spec_array_set_property(
    scope: &mut LocalScope,
    target: &Value,
    index: usize,
    value: PropertyValue,
) -> Result<(), Value> {
    // specialize array path
    if let Some(arr) = target.unpack().downcast_ref::<Array>(scope) {
        let mut inner = arr.items.borrow_mut();

        if let Ok(index) = u32::try_from(index) {
            if index < MAX_LENGTH {
                inner.set(index, value);
                return Ok(());
            }
        }
    }

    let index = scope.intern_usize(index);
    target.set_property(scope, index.into(), value)
}
