use std::any::Any;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::mem;

use dash_log::debug;
use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::gc::interner::sym;
use crate::localscope::LocalScope;
use crate::value::object::PropertyDataDescriptor;
use crate::{delegate, throw, Vm};

pub use self::holey::{Element, HoleyArray};

use super::object::{NamedObject, Object, PropertyKey, PropertyValue, PropertyValueKind};
use super::ops::conversions::ValueConversion;
use super::primitive::array_like_keys;
use super::root_ext::RootErrExt;
use super::{Root, Unrooted, Value};

mod holey;

pub const MAX_LENGTH: usize = 4294967295;

#[derive(Debug)]
pub enum ArrayInner<E> {
    NonHoley(Vec<E>),
    Holey(HoleyArray<E>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MaybeHoley<T> {
    Some(T),
    Hole,
}

impl<E: std::fmt::Debug> ArrayInner<E> {
    /// Computes the length.
    /// NOTE: this can potentially be an expensive operation for holey arrays, if it has an unusual high number of holes
    pub fn len(&self) -> usize {
        match self {
            ArrayInner::NonHoley(v) => v.len(),
            ArrayInner::Holey(v) => v.compute_len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ArrayInner::NonHoley(v) => v.is_empty(),
            ArrayInner::Holey(v) => v.is_empty(),
        }
    }

    pub fn get(&self, at: usize) -> Option<MaybeHoley<&E>> {
        match self {
            ArrayInner::NonHoley(v) => v.get(at).map(MaybeHoley::Some),
            ArrayInner::Holey(v) => v.get(at),
        }
    }

    /// Resizes this array, potentially switching to a holey kind.
    pub fn resize(&mut self, to: usize) {
        match self {
            ArrayInner::NonHoley(v) => {
                let len = v.len();
                if to <= len {
                    v.truncate(to);
                } else {
                    debug!("out of bounds resize, convert to holey array");
                    let hole = to - len;
                    let mut holey = mem::take(v).into_iter().map(Element::Value).collect::<Vec<_>>();
                    holey.push(Element::Hole { count: hole });
                    *self = ArrayInner::Holey(HoleyArray::from(holey));
                }
            }
            ArrayInner::Holey(v) => v.resize(to),
        }
    }

    pub fn set(&mut self, at: usize, value: E) {
        match self {
            ArrayInner::NonHoley(v) => {
                match at.cmp(&v.len()) {
                    Ordering::Less => v[at] = value,
                    Ordering::Equal => v.push(value),
                    Ordering::Greater => {
                        // resize us, causing self to have a hole and do the set logic below
                        self.resize(at);
                        self.set(at, value);
                        debug_assert!(matches!(self, Self::Holey(_)));
                    }
                }
            }
            ArrayInner::Holey(v) => {
                v.set(at, value);
            }
        }
    }

    pub fn push(&mut self, value: E) {
        match self {
            ArrayInner::NonHoley(v) => v.push(value),
            ArrayInner::Holey(v) => v.push(value),
        }
    }

    pub fn remove(&mut self, at: usize) -> Option<MaybeHoley<E>> {
        match self {
            ArrayInner::NonHoley(v) => {
                if at < v.len() {
                    Some(MaybeHoley::Some(v.remove(at)))
                } else {
                    None
                }
            }
            ArrayInner::Holey(v) => v.remove(at),
        }
    }
}

unsafe impl crate::gc::trace::Trace for ArrayInner<PropertyValue> {
    fn trace(&self, cx: &mut crate::gc::trace::TraceCtxt<'_>) {
        match self {
            ArrayInner::NonHoley(v) => v.trace(cx),
            ArrayInner::Holey(v) => v.trace(cx),
        }
    }
}

#[derive(Debug, Trace)]
pub struct Array {
    pub items: RefCell<ArrayInner<PropertyValue>>,
    obj: NamedObject,
}

fn get_named_object(vm: &Vm) -> NamedObject {
    NamedObject::with_prototype_and_constructor(vm.statics.array_prototype.clone(), vm.statics.array_ctor.clone())
}

impl Array {
    pub fn new(vm: &Vm) -> Self {
        Self::with_obj(get_named_object(vm))
    }

    /// Creates a non-holey array from a vec of values
    pub fn from_vec(vm: &Vm, items: Vec<PropertyValue>) -> Self {
        Self {
            items: RefCell::new(ArrayInner::NonHoley(items)),
            obj: get_named_object(vm),
        }
    }

    pub fn from_possibly_holey(vm: &Vm, elements: Vec<Element<PropertyValue>>) -> Self {
        Self {
            items: RefCell::new(ArrayInner::Holey(elements.into())),
            obj: get_named_object(vm),
        }
    }

    /// Creates a holey array with a given length
    pub fn with_hole(vm: &Vm, len: usize) -> Self {
        Self {
            items: RefCell::new(ArrayInner::Holey(HoleyArray::from(vec![Element::Hole { count: len }]))),
            obj: get_named_object(vm),
        }
    }

    /// Tries to convert this holey array into a non-holey array
    pub fn try_convert_to_non_holey(&self) {
        let values = if let ArrayInner::Holey(elements) = &mut *self.items.borrow_mut() {
            if !elements.has_hole() {
                mem::take(elements)
                    .into_inner()
                    .into_iter()
                    .map(|element| match element {
                        Element::Value(value) => value,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>()
            } else {
                return;
            }
        } else {
            return;
        };
        *self.items.borrow_mut() = ArrayInner::NonHoley(values);
    }

    /// Converts this potentially-holey array into a non-holey array, assuming that it succeeds.
    /// In other words, this assumes that there aren't any holes and change the kind to be non-holey.
    ///
    /// This can be useful to call after an operation that is guaranteed to remove any holes (e.g. filling an array)
    pub fn force_convert_to_non_holey(&self) {
        let values = if let ArrayInner::Holey(elements) = &mut *self.items.borrow_mut() {
            mem::take(elements)
                .into_inner()
                .into_iter()
                .map(|element| match element {
                    Element::Value(value) => value,
                    other => unreachable!("expected value element but got {other:?}"),
                })
                .collect::<Vec<_>>()
        } else {
            return;
        };
        *self.items.borrow_mut() = ArrayInner::NonHoley(values);
    }

    pub fn with_obj(obj: NamedObject) -> Self {
        Self {
            items: RefCell::new(ArrayInner::NonHoley(Vec::new())),
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

            if let Ok(index) = key.res(sc).parse::<usize>() {
                if index < MAX_LENGTH {
                    if let Some(element) = items.get(index) {
                        match element {
                            MaybeHoley::Some(v) => return Ok(Some(v.clone())),
                            MaybeHoley::Hole => return Ok(Some(PropertyValue::static_default(Value::undefined()))),
                        }
                    }
                }
            }
        }

        self.obj.get_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        if let PropertyKey::String(key) = &key {
            let mut items = self.items.borrow_mut();

            if key.sym() == sym::length {
                // TODO: this shouldnt be undefined
                let value = value.kind().get_or_apply(sc, Value::undefined()).root(sc)?;
                let new_len = value.to_number(sc)? as usize;

                if new_len > MAX_LENGTH {
                    throw!(sc, RangeError, "Invalid array length");
                }

                items.resize(new_len);
                return Ok(());
            }

            if let Ok(index) = key.res(sc).parse::<usize>() {
                if index < MAX_LENGTH {
                    items.set(index, value);
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

            if let Ok(index) = key.res(sc).parse::<usize>() {
                let mut items = self.items.borrow_mut();
                match items.remove(index) {
                    Some(MaybeHoley::Some(value)) => {
                        return value.get_or_apply(sc, Value::undefined()).root_err(sc);
                    }
                    Some(MaybeHoley::Hole) | None => return Ok(Value::undefined().into()),
                }
            }
        }

        self.obj.delete_property(sc, key)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: Handle,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(scope, callee, this, args)
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

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        let items = self.items.borrow();
        Ok(array_like_keys(sc, items.len()).collect())
    }
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
        callee: Handle,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ArrayIterator {
    pub fn new(sc: &mut LocalScope, value: Value) -> Result<Self, Value> {
        let length = value.length_of_array_like(sc)?;

        Ok(ArrayIterator {
            index: Cell::new(0),
            length,
            value,
            obj: NamedObject::with_prototype_and_constructor(
                sc.statics.array_iterator_prototype.clone(),
                sc.statics.object_ctor.clone(),
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
}

/// Equivalent to calling get_property, but specialized for arrays
pub fn spec_array_get_property(scope: &mut LocalScope, target: &Value, index: usize) -> Result<Unrooted, Unrooted> {
    if let Some(arr) = target.downcast_ref::<Array>() {
        let inner = arr.items.borrow();
        return match inner.get(index) {
            Some(MaybeHoley::Some(value)) => value.get_or_apply(scope, Value::undefined()),
            Some(MaybeHoley::Hole) | None => Ok(Value::undefined().into()),
        };
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
    if let Some(arr) = target.downcast_ref::<Array>() {
        let mut inner = arr.items.borrow_mut();

        if index < MAX_LENGTH {
            inner.set(index, value);
            return Ok(());
        }
    }

    let index = scope.intern_usize(index);
    target.set_property(scope, index.into(), value)
}

#[cfg(test)]
mod tests {
    use crate::value::array::holey::HoleyArray;
    use crate::value::array::{ArrayInner, Element, MaybeHoley};

    #[test]
    fn non_holey_get() {
        let arr = ArrayInner::NonHoley(vec![1, 2, 3]);
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&1)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Some(&2)));
        assert_eq!(arr.get(2), Some(MaybeHoley::Some(&3)));
    }

    #[test]
    fn holey_get() {
        let arr = ArrayInner::Holey(HoleyArray::from(vec![
            Element::Value(1),
            Element::Hole { count: 3 },
            Element::Value(2),
            Element::Hole { count: 5 },
            Element::Value(3),
            Element::Hole { count: 5 },
        ]));
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&1)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(2), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(3), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(4), Some(MaybeHoley::Some(&2)));
        assert_eq!(arr.get(5), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(6), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(7), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(8), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(9), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(10), Some(MaybeHoley::Some(&3)));
        assert_eq!(arr.get(13), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(17), None);
    }

    #[test]
    fn non_holey_resize() {
        let mut arr = ArrayInner::NonHoley(vec![1, 2, 3, 4]);
        arr.resize(2);
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&1)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Some(&2)));
        assert_eq!(arr.get(2), None);
        arr.resize(3);
        assert!(matches!(arr, ArrayInner::Holey(..)));
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&1)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Some(&2)));
        assert_eq!(arr.get(2), Some(MaybeHoley::Hole));
        arr.resize(0);
        assert_eq!(arr.get(0), None);
        arr.resize(1);
        assert_eq!(arr.get(0), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(1), None);
    }

    #[test]
    fn holey_resize() {
        let mut arr = ArrayInner::Holey(HoleyArray::from(vec![
            Element::Value(1),
            Element::Hole { count: 3 },
            Element::Value(2),
            Element::Hole { count: 5 },
            Element::Value(3),
            Element::Hole { count: 5 },
        ]));
        let len = arr.len();
        arr.resize(2);
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&1)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(2), None);
        for i in 3..len {
            assert_eq!(arr.get(i), None);
        }
        arr.resize(1000);
        assert_eq!(arr.get(1), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(999), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(1000), None);
    }

    #[test]
    fn holey_set() {
        let mut arr = ArrayInner::Holey(HoleyArray::from(vec![
            Element::Value(1),
            Element::Hole { count: 3 },
            Element::Value(2),
            Element::Hole { count: 5 },
            Element::Value(3),
            Element::Hole { count: 5 },
        ]));
        arr.set(0, 2);
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&2)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Hole));

        // in the middle of a 3-element hole should split it into (Hole(1), Some, Hole(1))
        arr.set(2, 3);
        assert_eq!(arr.get(2), Some(MaybeHoley::Some(&3)));
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&2)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(3), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(4), Some(MaybeHoley::Some(&2)));

        // arr is now [Some(2), Hole(1), Some(3)], check that setting a Hole(1) works
        arr.set(1, 4);
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&2)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Some(&4)));
        assert_eq!(arr.get(2), Some(MaybeHoley::Some(&3)));

        // setting at len
        let len = arr.len();
        assert_eq!(arr.get(len), None);
        arr.set(len, 999);
        assert_eq!(arr.get(len), Some(MaybeHoley::Some(&999)));

        // setting two past len should have a Hole(2) before it
        let len = arr.len();
        assert_eq!(arr.get(len + 2), None);
        arr.set(len + 2, 1000);
        assert_eq!(arr.get(len + 2), Some(MaybeHoley::Some(&1000)));
        assert_eq!(arr.get(len + 1), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(len), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(len + 3), None);
    }

    #[test]
    fn non_holey_set() {
        let mut arr = ArrayInner::NonHoley(vec![1, 2, 3]);
        arr.set(3, 4);
        assert!(matches!(arr, ArrayInner::NonHoley(_)));
        arr.set(5, 6);
        assert!(matches!(arr, ArrayInner::Holey(_)));
        assert_eq!(arr.get(5), Some(MaybeHoley::Some(&6)));
        assert_eq!(arr.get(4), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(6), None);
    }

    #[test]
    fn holey_remove() {
        let mut arr = ArrayInner::Holey(HoleyArray::from(vec![
            Element::Value(1),
            Element::Hole { count: 3 },
            Element::Value(2),
            Element::Hole { count: 5 },
            Element::Value(3),
            Element::Hole { count: 5 },
        ]));
        arr.remove(2);
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&1)));
        assert_eq!(arr.get(1), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(2), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(3), Some(MaybeHoley::Some(&2)));
        arr.remove(0);
        assert_eq!(arr.get(0), Some(MaybeHoley::Hole));
        assert_eq!(arr.get(1), Some(MaybeHoley::Hole));

        arr.remove(0);
        arr.remove(0);
        // Hole should now be gone
        assert_eq!(arr.get(0), Some(MaybeHoley::Some(&2)));
    }

    fn holey<T>(arr: &ArrayInner<T>) -> &[Element<T>] {
        match arr {
            ArrayInner::NonHoley(_) => unreachable!(),
            ArrayInner::Holey(h) => h.inner(),
        }
    }

    #[test]
    fn zero_sized_zoles() {
        // Tests match arms in HoleyArray::set
        let mut arr: ArrayInner<i32> = ArrayInner::Holey(HoleyArray::from(vec![Element::Hole { count: 1 }]));
        arr.set(0, 4);
        assert_eq!(holey(&arr), &[Element::Value(4)]);

        let mut arr: ArrayInner<i32> = ArrayInner::Holey(HoleyArray::from(vec![Element::Hole { count: 2 }]));
        arr.set(0, 4);
        assert_eq!(holey(&arr), &[Element::Value(4), Element::Hole { count: 1 }]);

        let mut arr: ArrayInner<i32> = ArrayInner::Holey(HoleyArray::from(vec![Element::Hole { count: 2 }]));
        arr.set(1, 4);
        assert_eq!(holey(&arr), &[Element::Hole { count: 1 }, Element::Value(4)]);

        let mut arr: ArrayInner<i32> = ArrayInner::Holey(HoleyArray::from(vec![Element::Hole { count: 3 }]));
        arr.set(1, 4);
        assert_eq!(
            holey(&arr),
            &[
                Element::Hole { count: 1 },
                Element::Value(4),
                Element::Hole { count: 1 }
            ]
        );
    }
}
