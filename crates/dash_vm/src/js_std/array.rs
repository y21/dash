use std::ops::Range;

use crate::localscope::LocalScope;
use crate::throw;
use crate::value::array;
use crate::value::array::Array;
use crate::value::array::ArrayIterator;
use crate::value::function::native::CallContext;
use crate::value::object::PropertyValue;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::ops::equality::ValueEquality;
use crate::value::Value;
use crate::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let array = Array::new(cx.scope);
    Ok(cx.scope.gc_mut().register(array).into())
}

fn join_inner(sc: &mut LocalScope, array: Value, separator: &str) -> Result<Value, Value> {
    let length = array.length_of_array_like(sc)?;

    let mut result = String::new();

    for i in 0..length {
        if i > 0 {
            result.push_str(separator);
        }

        let i = i.to_string();
        let element = array.get_property(sc, i.as_str().into())?;
        let s = element.to_string(sc)?;
        result.push_str(&s);
    }

    Ok(Value::String(result.into()))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    join_inner(cx.scope, cx.this, ",")
}

pub fn join(cx: CallContext) -> Result<Value, Value> {
    let sep = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    join_inner(cx.scope, cx.this, &sep)
}

pub fn values(cx: CallContext) -> Result<Value, Value> {
    let iter = ArrayIterator::new(cx.scope, cx.this)?;
    Ok(cx.scope.register(iter).into())
}

pub fn at(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)? as i64;
    let mut index = cx.args.first().unwrap_or_undefined().to_integer_or_infinity(cx.scope)? as i64;

    if index < 0 {
        index += len;
    }

    if index < 0 || index >= len {
        return Ok(Value::undefined());
    }

    let index = index.to_string();
    this.get_property(cx.scope, index.as_str().into())
}

pub fn concat(cx: CallContext) -> Result<Value, Value> {
    let _this = Value::Object(cx.this.to_object(cx.scope)?);
    let mut array = Vec::new();
    // TODO: add elements from `this` to `array`

    for arg in &cx.args {
        let len = arg.length_of_array_like(cx.scope)?;
        for i in 0..len {
            let i = i.to_string();
            let element = arg.get_property(cx.scope, i.as_str().into())?;
            array.push(PropertyValue::static_default(element));
        }
    }

    let array = Array::from_vec(cx.scope, array);

    Ok(cx.scope.register(array).into())
}

pub fn entries(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Not implemented")
}

pub fn keys(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Not implemented")
}

pub fn every(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv, Value::number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;
        if !test {
            return Ok(false.into());
        }
    }

    Ok(true.into())
}

pub fn some(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv, Value::number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;
        if test {
            return Ok(true.into());
        }
    }

    Ok(false.into())
}

pub fn fill(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let value = cx.args.first().unwrap_or_undefined();

    for i in 0..len {
        let pk = i.to_string();
        this.set_property(cx.scope, pk.into(), PropertyValue::static_default(value.clone()))?;
    }

    Ok(this)
}

pub fn filter(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();
    let mut values = Vec::new();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv.clone(), Value::number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;

        if test {
            cx.scope.add_value(pkv.clone());
            values.push(PropertyValue::static_default(pkv));
        }
    }

    let values = Array::from_vec(cx.scope, values);

    Ok(cx.scope.register(values).into())
}

pub fn reduce(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();
    let initial_value = cx.args.get(1);

    let (start, mut accumulator) = match (len, initial_value) {
        (0, None) => throw!(cx.scope, TypeError, "Reduce of empty array with no initial value"),
        (0, Some(_)) => return Ok(initial_value.unwrap().clone()),
        (_, Some(initial)) => (0, initial.clone()),
        (1, None) => {
            let pk = 0.to_string();
            let pkv = this.get_property(cx.scope, pk.as_str().into())?;
            return Ok(pkv);
        }
        (_, None) => {
            let pkv = this.get_property(cx.scope, "0".into())?;
            let pkv2 = this.get_property(cx.scope, "1".into())?;
            let args = vec![pkv, pkv2, Value::number(1_f64)];
            (2, callback.apply(cx.scope, Value::undefined(), args)?)
        }
    };

    for k in start..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![accumulator, pkv, Value::number(k as f64)];
        accumulator = callback.apply(cx.scope, Value::undefined(), args)?;
    }

    Ok(accumulator)
}

pub fn find(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv.clone(), Value::number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;

        if test {
            return Ok(pkv);
        }
    }

    Ok(Value::undefined())
}

pub fn find_index(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv, Value::number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;

        if test {
            return Ok(Value::number(k as f64));
        }
    }

    Ok(Value::number(-1.0))
}

pub fn flat(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Not implemented")
}

pub fn for_each(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv, Value::number(k as f64)];
        callback.apply(cx.scope, Value::undefined(), args)?;
    }

    Ok(Value::undefined())
}

pub fn includes(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let search_element = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        if pkv.strict_eq(&search_element, cx.scope)?.is_truthy() {
            return Ok(true.into());
        }
    }

    Ok(false.into())
}

pub fn index_of(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let search_element = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        if pkv.strict_eq(&search_element, cx.scope)?.is_truthy() {
            return Ok(Value::number(k as f64));
        }
    }

    Ok(Value::number(-1.0))
}

pub fn last_index_of(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let search_element = cx.args.first().unwrap_or_undefined();
    let from_index = cx
        .args
        .get(1)
        .map(|x| x.to_length_u(cx.scope))
        .transpose()?
        .unwrap_or(len);

    for k in (0..from_index).rev() {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        if pkv.strict_eq(&search_element, cx.scope)?.is_truthy() {
            return Ok(Value::number(k as f64));
        }
    }

    Ok(Value::number(-1.0))
}

pub fn map(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();
    let mut values = Vec::new();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv.clone(), Value::number(k as f64)];
        let value = callback.apply(cx.scope, Value::undefined(), args)?;

        cx.scope.add_value(value.clone());
        values.push(PropertyValue::static_default(value));
    }

    let values = Array::from_vec(cx.scope, values);

    Ok(cx.scope.register(values).into())
}

pub fn pop(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    if len == 0 {
        return Ok(Value::undefined());
    }

    let new_len = len - 1;
    let value = this
        .delete_property(cx.scope, new_len.to_string().into())?
        .root(cx.scope);
    this.set_property(
        cx.scope,
        "length".into(),
        PropertyValue::static_default(Value::number(new_len as f64)),
    )?;

    Ok(value)
}

pub fn push(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    let mut last = Value::undefined();

    if cx.args.is_empty() {
        this.set_property(
            cx.scope,
            len.to_string().into(),
            PropertyValue::static_default(Value::undefined()),
        )?;
    }

    for (idx, arg) in cx.args.into_iter().enumerate() {
        last = arg.clone();
        array::spec_array_set_property(cx.scope, &this, idx + len, PropertyValue::static_default(arg))?;
    }

    Ok(last)
}

pub fn reverse(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    // Strategy: Given [1,2,3,4,5], swap `i` with `len - i - 1` for every index `i` in `0..len / 2`
    for k in 0..len / 2 {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let pk2 = (len - k - 1).to_string();
        let pk2v = this.get_property(cx.scope, pk2.as_str().into())?;
        this.set_property(cx.scope, pk.into(), PropertyValue::static_default(pk2v))?;
        this.set_property(cx.scope, pk2.into(), PropertyValue::static_default(pkv))?;
    }

    Ok(this)
}

pub fn shift(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    if len == 0 {
        return Ok(Value::undefined());
    }

    let prop = this.delete_property(cx.scope, "0".into())?.root(cx.scope);

    for k in 1..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        this.set_property(cx.scope, (k - 1).to_string().into(), PropertyValue::static_default(pkv))?;
    }

    this.set_property(
        cx.scope,
        "length".into(),
        PropertyValue::static_default(Value::number((len - 1) as f64)),
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
    let range = range.start as isize..range.end as isize;

    let new_len = (range.end + shift_by) as usize;

    // If the range end + shift_by results in a value greater than the length of the array, we need to
    // set the length of the array to the new length.
    // Technically this isn't needed, and we can just let the array grow as needed, but this is for clarity
    if range.end + shift_by > len as isize {
        arr.set_property(
            scope,
            "length".into(),
            PropertyValue::static_default(Value::number(new_len as f64)),
        )?;
    }

    // Start shifting the elements by the shift_by (can be either negative or positive) amount
    for k in range {
        let pk = k.to_string();
        let pkv = arr.get_property(scope, pk.as_str().into())?;
        arr.set_property(
            scope,
            (k + shift_by).to_string().into(),
            PropertyValue::static_default(pkv),
        )?;
    }

    // If the shift_by is negative, we need to delete the remaining elements at the end that were shifted
    // This must be done after the shifting, otherwise we would be deleting elements before they can be shifted
    if shift_by < 0 {
        arr.set_property(
            scope,
            "length".into(),
            PropertyValue::static_default(Value::number(new_len as f64)),
        )?;
    }

    Ok(())
}

pub fn unshift(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let arg_len = cx.args.len();
    let new_len = len + cx.args.len();

    shift_array(cx.scope, &this, len, arg_len as isize, 0..len)?;

    for (idx, arg) in cx.args.into_iter().enumerate() {
        this.set_property(cx.scope, idx.to_string().into(), PropertyValue::static_default(arg))?;
    }

    Ok(Value::number(new_len as f64))
}

fn to_slice_index(index: isize, len: usize) -> usize {
    if index < 0 {
        let new_index = len as isize + index;
        if new_index < 0 {
            0
        } else {
            new_index as usize
        }
    } else {
        index as usize
    }
}

pub fn slice(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    let start = match cx.args.get(0) {
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
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        values.push(PropertyValue::static_default(pkv));
    }

    let values = Array::from_vec(cx.scope, values);

    Ok(cx.scope.register(values).into())
}

pub fn from(cx: CallContext) -> Result<Value, Value> {
    fn with_iterator(scope: &mut LocalScope, items: Value, mapper: Option<Value>) -> Result<Value, Value> {
        let mut values = Vec::new();

        let next = items.get_property(scope, "next".into())?;
        loop {
            let item = next.apply(scope, items.clone(), Vec::new())?;
            let done = matches!(item.get_property(scope, "done".into())?, Value::Boolean(true));
            if done {
                break;
            }
            let value = item.get_property(scope, "value".into())?;
            let value = match &mapper {
                Some(mapper) => mapper.apply(scope, Value::undefined(), vec![value])?,
                None => value,
            };
            values.push(PropertyValue::static_default(value));
        }

        let values = Array::from_vec(scope, values);
        Ok(Value::Object(scope.register(values)))
    }

    fn with_array_like(scope: &mut LocalScope, items: Value, mapper: Option<Value>) -> Result<Value, Value> {
        let len = items.length_of_array_like(scope)?;

        let mut values = Vec::new();

        for i in 0..len {
            let value = items.get_property(scope, i.to_string().into())?;
            let value = match &mapper {
                Some(mapper) => mapper.apply(scope, Value::undefined(), vec![value])?,
                None => value,
            };
            values.push(PropertyValue::static_default(value));
        }

        let values = Array::from_vec(scope, values);
        Ok(Value::Object(scope.register(values)))
    }

    let mut args = cx.args.into_iter();

    let items = args.next().unwrap_or_undefined();
    let mapper = args.next();

    let items_iterator = {
        let iterator = cx.scope.statics.symbol_iterator.clone();
        items.get_property(cx.scope, iterator.into())?.into_option()
    };

    match items_iterator {
        Some(iterator) => {
            let iterator = iterator.apply(cx.scope, items, Vec::new())?;
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
    let this = Value::Object(cx.this.to_object(cx.scope)?);
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
            let previous = this.get_property(cx.scope, (j - 1).to_string().into())?;
            let current = this.get_property(cx.scope, j.to_string().into())?;
            let ordering = compare_fn
                .apply(cx.scope, Value::undefined(), vec![previous.clone(), current.clone()])?
                .to_int32(cx.scope)?;

            if ordering > 0 {
                this.set_property(
                    cx.scope,
                    (j - 1).to_string().into(),
                    PropertyValue::static_default(current),
                )?;
                this.set_property(cx.scope, j.to_string().into(), PropertyValue::static_default(previous))?;
            } else {
                break;
            }
        }
    }

    Ok(cx.this)
}
