use ControlFlow::{Break, Continue};
use std::cmp;
use std::convert::Infallible;
use std::ops::{ControlFlow, Range};

use crate::frame::This;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::array::{Array, ArrayIterator, require_valid_array_length};
use crate::value::function::args::CallArgs;
use crate::value::function::native::CallContext;
use crate::value::object::{OrdObject, Object as _, PropertyValue};
use crate::value::ops::conversions::ValueConversion;
use crate::value::ops::equality::strict_eq;
use crate::value::propertykey::ToPropertyKey;
use crate::value::root_ext::RootErrExt;
use crate::value::string::JsString;
use crate::value::{Root, Unpack, Value, ValueContext, array};
use dash_middle::interner::sym;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let size = cx.args.first().unwrap_or_undefined().to_length_u(cx.scope)?;
    let obj = OrdObject::instance_for_new_target(cx.new_target.unwrap_or(cx.scope.statics.array_ctor), cx.scope)?;
    let array = Array::with_hole(size, obj);
    Ok(cx.scope.register(array).into())
}

fn wrapping_index_val(start: Value, cx: &mut CallContext, len: usize) -> Result<usize, Value> {
    let start = start.to_integer_or_infinity(cx.scope)?;
    Ok(if start == f64::NEG_INFINITY {
        0
    } else if start < 0.0 {
        cmp::max(len as isize + start as isize, 0) as usize
    } else {
        cmp::min(start as isize, len as isize) as usize
    })
}

fn join_inner(sc: &mut LocalScope, array: Value, separator: JsString) -> Result<Value, Value> {
    let length = array.length_of_array_like(sc)?;

    let mut result = String::new();

    for i in 0..length {
        if i > 0 {
            result.push_str(separator.res(sc));
        }

        let element = array.get_property(i.to_key(sc), sc).root(sc)?;
        if !element.is_nullish() {
            let s = element.to_js_string(sc)?;
            result.push_str(s.res(sc));
        }
    }

    Ok(Value::string(sc.intern(result).into()))
}

fn for_each_element<B>(
    scope: &mut LocalScope<'_>,
    this: Value,
    mut f: impl FnMut(&mut LocalScope<'_>, Value, Value) -> Result<ControlFlow<B>, Value>,
) -> Result<ControlFlow<B>, Value> {
    let len = this.length_of_array_like(scope)?;
    for k in 0..len {
        if let Some(value) = this.get_property_descriptor(k.to_key(scope), scope).root_err(scope)? {
            let value = value.get_or_apply(scope, This::Bound(this)).root(scope)?;
            if let Break(value) = f(scope, value, Value::number(k as f64))? {
                return Ok(Break(value));
            }
        }
    }

    Ok(Continue(()))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    join_inner(cx.scope, cx.this, sym::comma.into())
}

pub fn join(cx: CallContext) -> Result<Value, Value> {
    let sep = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    join_inner(cx.scope, cx.this, sep)
}

pub fn values(cx: CallContext) -> Result<Value, Value> {
    let iter = ArrayIterator::new(cx.scope, cx.this)?;
    Ok(cx.scope.register(iter).into())
}

pub fn at(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)? as i64;
    let mut index = cx.args.first().unwrap_or_undefined().to_integer_or_infinity(cx.scope)? as i64;

    if index < 0 {
        index += len;
    }

    if index < 0 || index >= len {
        return Ok(Value::undefined());
    }

    this.get_property((index as usize).to_key(cx.scope), cx.scope)
        .root(cx.scope)
}

pub fn concat(cx: CallContext) -> Result<Value, Value> {
    let _this = Value::object(cx.this.to_object(cx.scope)?);
    let mut array = Vec::new();
    // TODO: add elements from `this` to `array`

    for arg in &cx.args {
        let len = arg.length_of_array_like(cx.scope)?;
        for i in 0..len {
            let element = arg.get_property(i.to_key(cx.scope), cx.scope).root(cx.scope)?;
            array.push(PropertyValue::static_default(element));
        }
    }

    let array = Array::from_vec(array, cx.scope);

    Ok(cx.scope.register(array).into())
}

pub fn entries(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Not implemented")
}

pub fn keys(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Not implemented")
}

pub fn every(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let callback = cx.args.first().unwrap_or_undefined();
    let cb_this = match cx.args.get(1) {
        Some(v) => Value::object(v.to_object(cx.scope)?),
        None => Value::undefined(),
    };

    let all_true = for_each_element(cx.scope, this, |scope, elem, idx| {
        if callback
            .apply(This::Bound(cb_this), [elem, idx, this].into(), scope)
            .root(scope)?
            .to_boolean(scope)?
        {
            Ok(Continue(()))
        } else {
            Ok(Break(()))
        }
    })?
    .is_continue();

    Ok(Value::boolean(all_true))
}

pub fn some(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let callback = cx.args.first().unwrap_or_undefined();
    let cb_this = match cx.args.get(1) {
        Some(v) => Value::object(v.to_object(cx.scope)?),
        None => Value::undefined(),
    };

    let any_true = for_each_element(cx.scope, this, |scope, elem, idx| {
        if callback
            .apply(This::Bound(cb_this), [elem, idx, this].into(), scope)
            .root(scope)?
            .to_boolean(scope)?
        {
            Ok(Break(()))
        } else {
            Ok(Continue(()))
        }
    })?
    .is_break();

    Ok(Value::boolean(any_true))
}

pub fn fill(mut cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let value = cx.args.first().unwrap_or_undefined();

    let k = wrapping_index_val(cx.args.get(1).unwrap_or_undefined(), &mut cx, len)?;

    let relative_end = cx
        .args
        .get(2)
        .map(|v| v.to_integer_or_infinity(cx.scope))
        .transpose()?
        .unwrap_or(len as f64);

    let final_ = if relative_end == f64::NEG_INFINITY {
        0
    } else if relative_end < 0.0 {
        cmp::max(len as isize + relative_end as isize, 0) as usize
    } else {
        cmp::min(relative_end as isize, len as isize) as usize
    };

    for i in k..final_ {
        array::spec_array_set_property(cx.scope, &this, i, PropertyValue::static_default(value))?;
    }

    if let Some(arr) = this.unpack().downcast_ref::<Array>(cx.scope) {
        arr.try_convert_to_non_holey();
    }

    Ok(this)
}

pub fn filter(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let callback = cx.args.first().unwrap_or_undefined();
    let cb_this = match cx.args.get(1) {
        Some(v) => Value::object(v.to_object(cx.scope)?),
        None => Value::undefined(),
    };
    let mut values = Vec::new();

    let (Break(()) | Continue(())) = for_each_element(cx.scope, this, |scope, elem, idx| {
        if callback
            .apply(This::Bound(cb_this), [elem, idx, this].into(), scope)
            .root(scope)?
            .to_boolean(scope)?
        {
            values.push(PropertyValue::static_default(elem));
        }
        Ok(Continue(()))
    })?;

    let values = Array::from_vec(values, cx.scope);

    Ok(cx.scope.register(values).into())
}

pub fn reduce(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();
    let initial_value = cx.args.get(1).copied();

    let (start, mut accumulator) = match (len, initial_value) {
        (0, None) => throw!(cx.scope, TypeError, "Reduce of empty array with no initial value"),
        (0, Some(_)) => return Ok(initial_value.unwrap()),
        (_, Some(initial)) => (0, initial),
        (1, None) => {
            let pkv = this.get_property(sym::zero.to_key(cx.scope), cx.scope).root(cx.scope)?;
            return Ok(pkv);
        }
        (_, None) => {
            let pkv = this.get_property(sym::zero.to_key(cx.scope), cx.scope).root(cx.scope)?;
            let pkv2 = this.get_property(sym::one.to_key(cx.scope), cx.scope).root(cx.scope)?;
            let args = [pkv, pkv2, Value::number(1_f64)].into();
            (2, callback.apply(This::Default, args, cx.scope).root(cx.scope)?)
        }
    };

    for k in start..len {
        let pkv = this.get_property(k.to_key(cx.scope), cx.scope).root(cx.scope)?;
        let args = [accumulator, pkv, Value::number(k as f64), this].into();
        accumulator = callback.apply(This::Default, args, cx.scope).root(cx.scope)?;
    }

    Ok(accumulator)
}

pub fn find(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let callback = cx.args.first().unwrap_or_undefined();
    let cb_this = match cx.args.get(1) {
        Some(v) => Value::object(v.to_object(cx.scope)?),
        None => Value::undefined(),
    };

    let element = for_each_element(cx.scope, this, |scope, elem, idx| {
        if callback
            .apply(This::Bound(cb_this), [elem, idx, this].into(), scope)
            .root(scope)?
            .to_boolean(scope)?
        {
            Ok(Break(elem))
        } else {
            Ok(Continue(()))
        }
    })?;

    Ok(match element {
        Break(value) => value,
        Continue(()) => Value::undefined(),
    })
}

pub fn find_index(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let callback = cx.args.first().unwrap_or_undefined();
    let cb_this = match cx.args.get(1) {
        Some(v) => Value::object(v.to_object(cx.scope)?),
        None => Value::undefined(),
    };

    let element = for_each_element(cx.scope, this, |scope, elem, idx| {
        if callback
            .apply(This::Bound(cb_this), [elem, idx, this].into(), scope)
            .root(scope)?
            .to_boolean(scope)?
        {
            Ok(Break(idx))
        } else {
            Ok(Continue(()))
        }
    })?;

    Ok(match element {
        Break(value) => value,
        Continue(()) => Value::number(-1.0),
    })
}

pub fn flat(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Not implemented")
}

pub fn for_each(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let callback = cx.args.first().unwrap_or_undefined();
    let cb_this = match cx.args.get(1) {
        Some(v) => Value::object(v.to_object(cx.scope)?),
        None => Value::undefined(),
    };

    let (Break(()) | Continue(())) = for_each_element(cx.scope, this, |scope, elem, idx| {
        callback
            .apply(This::Bound(cb_this), [elem, idx, this].into(), scope)
            .root_err(scope)?;
        Ok(Continue(()))
    })?;

    Ok(Value::undefined())
}

pub fn includes(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let search_element = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pkv = this.get_property(k.to_key(cx.scope), cx.scope).root(cx.scope)?;
        if strict_eq(pkv, search_element) {
            return Ok(true.into());
        }
    }

    Ok(false.into())
}

pub fn index_of(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    if len == 0 {
        return Ok(Value::number(-1.));
    }

    let search_element = cx.args.first().unwrap_or_undefined();
    let from_index = cx.args.get(1).unwrap_or_undefined().to_integer_or_infinity(cx.scope)?;
    if from_index == f64::INFINITY {
        return Ok(Value::number(-1.));
    } else if from_index == f64::NEG_INFINITY {
        return Ok(Value::number(0.));
    }
    let from_index = if from_index.is_sign_positive() {
        from_index as usize
    } else {
        let k = len as isize + from_index as isize;
        usize::try_from(k).unwrap_or_default()
    };

    for k in from_index..len {
        let pkv = this.get_property(k.to_key(cx.scope), cx.scope).root(cx.scope)?;
        if strict_eq(pkv, search_element) {
            return Ok(Value::number(k as f64));
        }
    }

    Ok(Value::number(-1.0))
}

pub fn last_index_of(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    if len == 0 {
        return Ok(Value::number(-1.));
    }

    let search_element = cx.args.first().unwrap_or_undefined();
    let from_index = if let Some(from_index) = cx.args.get(1) {
        from_index.to_integer_or_infinity(cx.scope)?
    } else {
        -1.
    };
    let from_index = if from_index == f64::NEG_INFINITY {
        return Ok(Value::number(-1.));
    } else if from_index.is_sign_positive() {
        (from_index as usize).min(len - 1)
    } else {
        usize::try_from(len as isize + from_index as isize).unwrap_or_default()
    };

    for k in (0..=from_index).rev() {
        let pkv = this.get_property(k.to_key(cx.scope), cx.scope).root(cx.scope)?;
        if strict_eq(pkv, search_element) {
            return Ok(Value::number(k as f64));
        }
    }

    Ok(Value::number(-1.0))
}

pub fn map(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let callback = cx.args.first().unwrap_or_undefined();
    let cb_this = match cx.args.get(1) {
        Some(v) => Value::object(v.to_object(cx.scope)?),
        None => Value::undefined(),
    };
    let mut values = Vec::new();

    let (Break(()) | Continue(())) = for_each_element(cx.scope, this, |scope, elem, idx| {
        let mapped = callback
            .apply(This::Bound(cb_this), [elem, idx, this].into(), scope)
            .root(scope)?;
        values.push(PropertyValue::static_default(mapped));
        Ok(Continue(()))
    })?;

    let values = Array::from_vec(values, cx.scope);

    Ok(cx.scope.register(values).into())
}

pub fn pop(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    if len == 0 {
        return Ok(Value::undefined());
    }

    let new_len = len - 1;
    let value = this.delete_property(new_len.to_key(cx.scope), cx.scope)?.root(cx.scope);
    this.set_property(
        sym::length.to_key(cx.scope),
        PropertyValue::static_default(Value::number(new_len as f64)),
        cx.scope,
    )?;

    Ok(value)
}

pub fn push(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    let mut last = Value::undefined();

    if cx.args.is_empty() {
        this.set_property(
            len.to_key(cx.scope),
            PropertyValue::static_default(Value::undefined()),
            cx.scope,
        )?;
    }

    for (idx, arg) in cx.args.into_iter().enumerate() {
        last = arg;
        array::spec_array_set_property(cx.scope, &this, idx + len, PropertyValue::static_default(arg))?;
    }

    Ok(last)
}

pub fn reverse(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    // Strategy: Given [1,2,3,4,5], swap `i` with `len - i - 1` for every index `i` in `0..len / 2`
    for k in 0..len / 2 {
        let pkv = this.get_property(k.to_key(cx.scope), cx.scope).root(cx.scope)?;
        let k2 = len - k - 1;
        let pk2v = this.get_property(k2.to_key(cx.scope), cx.scope).root(cx.scope)?;
        this.set_property(k.to_key(cx.scope), PropertyValue::static_default(pk2v), cx.scope)?;
        this.set_property(k2.to_key(cx.scope), PropertyValue::static_default(pkv), cx.scope)?;
    }

    Ok(this)
}

pub fn shift(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    if len == 0 {
        return Ok(Value::undefined());
    }

    let prop = this
        .delete_property(sym::zero.to_key(cx.scope), cx.scope)?
        .root(cx.scope);

    for k in 1..len {
        let prev_pk = k - 1;
        let pkv = this.get_property(k.to_key(cx.scope), cx.scope).root(cx.scope)?;
        this.set_property(prev_pk.to_key(cx.scope), PropertyValue::static_default(pkv), cx.scope)?;
    }

    this.set_property(
        sym::length.to_key(cx.scope),
        PropertyValue::static_default(Value::number((len - 1) as f64)),
        cx.scope,
    )?;

    Ok(prop)
}

/// Shifts the elements of a JavaScript array by a given amount to the left (negative value) or right (positive value).
fn shift_array(
    scope: &mut LocalScope,
    arr: &Value,
    len: usize,
    shift_by: isize,
    range: Range<usize>,
) -> Result<(), Value> {
    if shift_by == 0 {
        // No shifting needs to happen (and can't, this short circuit is required by splice)
        return Ok(());
    }

    let range = range.start as isize..range.end as isize;

    let new_len = (range.end + shift_by) as usize;

    // If the range end + shift_by results in a value greater than the length of the array, we need to
    // set the length of the array to the new length.
    // Technically this isn't needed, and we can just let the array grow as needed, but this is for clarity
    if range.end + shift_by > len as isize {
        arr.set_property(
            sym::length.to_key(scope),
            PropertyValue::static_default(Value::number(new_len as f64)),
            scope,
        )?;
    }

    // Start shifting the elements by the shift_by (can be either negative or positive) amount
    for k in range {
        let pk = scope.intern_isize(k);
        let shift_pk = scope.intern_isize(k + shift_by);
        let pkv = arr.get_property(pk.to_key(scope), scope).root(scope)?;
        arr.set_property(shift_pk.to_key(scope), PropertyValue::static_default(pkv), scope)?;
    }

    // If the shift_by is negative, we need to delete the remaining elements at the end that were shifted
    // This must be done after the shifting, otherwise we would be deleting elements before they can be shifted
    if shift_by < 0 {
        arr.set_property(
            sym::length.to_key(scope),
            PropertyValue::static_default(Value::number(new_len as f64)),
            scope,
        )?;
    }

    Ok(())
}

pub fn unshift(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let arg_len = cx.args.len();
    let new_len = len + cx.args.len();

    shift_array(cx.scope, &this, len, arg_len as isize, 0..len)?;

    for (idx, arg) in cx.args.into_iter().enumerate() {
        this.set_property(idx.to_key(cx.scope), PropertyValue::static_default(arg), cx.scope)?;
    }

    Ok(Value::number(new_len as f64))
}

fn to_slice_index(index: isize, len: usize) -> usize {
    if index < 0 {
        let new_index = len as isize + index;
        if new_index < 0 { 0 } else { new_index as usize }
    } else {
        index as usize
    }
}

pub fn slice(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    let start = match cx.args.first() {
        Some(v) => to_slice_index(v.to_int32(cx.scope)? as isize, len),
        None => 0,
    };

    let end = match cx.args.get(1) {
        Some(v) => to_slice_index(v.to_int32(cx.scope)? as isize, len),
        None => len,
    };

    // TODO: optimization opportunity to have a `SliceArray` type of internal array kind
    // that just stores a reference to the original array and the start/end indices
    // instead of allocating a new array for the subslice
    let mut values = Vec::new();

    for k in start..end {
        let pkv = this.get_property(k.to_key(cx.scope), cx.scope).root(cx.scope)?;
        values.push(PropertyValue::static_default(pkv));
    }

    let values = Array::from_vec(values, cx.scope);

    Ok(cx.scope.register(values).into())
}

pub fn is_array(cx: CallContext) -> Result<Value, Value> {
    Ok(Value::boolean(
        cx.args
            .first()
            .unwrap_or_undefined()
            .unpack()
            .downcast_ref::<Array>(cx.scope)
            .is_some(),
    ))
}

pub fn for_each_js_iterator_element<B, F: FnMut(&mut LocalScope<'_>, Value) -> Result<ControlFlow<B>, Value>>(
    scope: &mut LocalScope<'_>,
    iter: Value,
    mut f: F,
) -> Result<ControlFlow<B>, Value> {
    let next = iter.get_property(sym::next.to_key(scope), scope).root(scope)?;
    loop {
        let item = next.apply(This::Bound(iter), CallArgs::empty(), scope).root(scope)?;
        let done = item
            .get_property(sym::done.to_key(scope), scope)
            .root(scope)?
            .is_truthy(scope);
        if done {
            break;
        }
        let value = item.get_property(sym::value.to_key(scope), scope).root(scope)?;
        if let Break(val) = f(scope, value)? {
            return Ok(Break(val));
        }
    }

    Ok(Continue(()))
}

pub fn from(cx: CallContext) -> Result<Value, Value> {
    fn with_iterator(scope: &mut LocalScope, items: Value, mapper: Option<Value>) -> Result<Value, Value> {
        let mut values = Vec::new();

        for_each_js_iterator_element(scope, items, |scope, value| {
            let value = match &mapper {
                Some(mapper) => mapper.apply(This::Default, [value].into(), scope).root(scope)?,
                None => value,
            };
            values.push(PropertyValue::static_default(value));
            Ok(ControlFlow::<Infallible, _>::Continue(()))
        })?;

        let values = Array::from_vec(values, scope);
        Ok(Value::object(scope.register(values)))
    }

    fn with_array_like(scope: &mut LocalScope, items: Value, mapper: Option<Value>) -> Result<Value, Value> {
        let len = items.length_of_array_like(scope)?;

        let mut values = Vec::new();

        for i in 0..len {
            let value = items.get_property(i.to_key(scope), scope).root(scope)?;
            let value = match &mapper {
                Some(mapper) => mapper.apply(This::Default, [value].into(), scope).root(scope)?,
                None => value,
            };
            values.push(PropertyValue::static_default(value));
        }

        let values = Array::from_vec(values, scope);
        Ok(Value::object(scope.register(values)))
    }

    let mut args = cx.args.into_iter();

    let items = args.next().unwrap_or_undefined();
    let mapper = args.next();

    let items_iterator = {
        let iterator = cx.scope.statics.symbol_iterator;
        items
            .get_property(iterator.to_key(cx.scope), cx.scope)
            .root(cx.scope)?
            .into_option()
    };

    match items_iterator {
        Some(iterator) => {
            let iterator = iterator
                .apply(This::Bound(items), CallArgs::empty(), cx.scope)
                .root(cx.scope)?;
            with_iterator(cx.scope, iterator, mapper)
        }
        None => with_array_like(cx.scope, items, mapper),
    }
}

// This implements an insertion sort for now since it's simple and okay for small arrays.
// We can always improve it later.
// It's worth noting that we unfortunately cannot use the sorting algorithm in the standard library,
// since that must happen in a closure that needs to return an `Ordering`, without the ability to
// return errors, but calling into JS can throw exceptions.
pub fn sort(cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    let Some(compare_fn) = cx.args.first().cloned() else {
        throw!(
            cx.scope,
            Error,
            "Array.prototype.sort currently requires an explicit compare function, try `.sort((a, b) => a - b)` to sort from lowest to highest"
        );
    };

    for i in 1..len {
        for j in (1..=i).rev() {
            let idx = j;
            let prev_idx = j - 1;

            let previous = this.get_property(prev_idx.to_key(cx.scope), cx.scope).root(cx.scope)?;
            let current = this.get_property(idx.to_key(cx.scope), cx.scope).root(cx.scope)?;
            let ordering = compare_fn
                .apply(This::Default, [previous, current].into(), cx.scope)
                .root(cx.scope)?
                .to_int32(cx.scope)?;

            if ordering > 0 {
                this.set_property(
                    prev_idx.to_key(cx.scope),
                    PropertyValue::static_default(current),
                    cx.scope,
                )?;
                this.set_property(idx.to_key(cx.scope), PropertyValue::static_default(previous), cx.scope)?;
            } else {
                break;
            }
        }
    }

    Ok(cx.this)
}

pub fn splice(mut cx: CallContext) -> Result<Value, Value> {
    let this = Value::object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    let start = wrapping_index_val(cx.args.first().unwrap_or_undefined(), &mut cx, len)?;
    let delete_count = match *cx.args {
        // 8. If start is not present, then
        [] => 0,
        // 9. Else if deleteCount is not present, then
        [_] => len - start,
        // 10. Else, ...
        [_, v, ..] => isize::clamp(v.to_integer_or_infinity(cx.scope)? as isize, 0, (len - start) as isize) as usize,
    };
    let item_count = cx.args.len().saturating_sub(2);

    // TODO: often the returned array is unused; it may be possible to pass a "return value is used" flag to CallContexts

    require_valid_array_length(cx.scope, delete_count)?;
    let mut values = Vec::with_capacity(delete_count);
    for k in 0..delete_count {
        let from = start + k;

        if let Some(delete_value) = this
            .get_property_descriptor(from.to_key(cx.scope), cx.scope)
            .root_err(cx.scope)?
        {
            values.push(delete_value);
        }
    }

    if item_count <= delete_count {
        // Since we delete more than we insert, overwrite elements at the delete index
        for (i, value) in cx.args.iter().skip(2).enumerate() {
            let i = i + start;
            this.set_property(i.to_key(cx.scope), PropertyValue::static_default(*value), cx.scope)?;
        }

        // Now shift the rest to the left and update the length
        shift_array(
            cx.scope,
            &this,
            len,
            -(delete_count as isize - item_count as isize),
            start + delete_count..len,
        )?;
    } else {
        let items_to_insert = item_count - delete_count;
        require_valid_array_length(cx.scope, len + items_to_insert)?;

        // We can overwrite `delete_count` number of items without shifting to the right
        // IOW, we only need to shift by `item_count - delete_count` to the right
        shift_array(
            cx.scope,
            &this,
            len,
            items_to_insert as isize,
            start + delete_count..len,
        )?;

        for (i, value) in cx.args.iter().skip(2).enumerate() {
            let i = i + start;
            this.set_property(i.to_key(cx.scope), PropertyValue::static_default(*value), cx.scope)?;
        }
    }

    Ok(cx.scope.register(Array::from_vec(values, cx.scope)).into())
}
