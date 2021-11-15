use std::borrow::Cow;

use crate::{
    gc::Handle,
    js_std,
    vm::{
        abstractions,
        value::{
            array::Array,
            function::{CallContext, NativeFunctionCallbackResult},
            object::{ExoticObject, Object},
            Value, ValueKind,
        },
        VM,
    },
};

use super::{
    error::{self},
    todo,
};

/// An array-like value that can be iterated over
pub struct ArrayLikeIterable {
    /// The iterable value
    pub value: Value,
    /// Current index
    pub index: usize,
}

impl ArrayLikeIterable {
    /// Creates a new array like iterable given an [ArrayLikeKind]
    pub fn new(value: Value) -> Self {
        Self { value, index: 0 }
    }

    /// Yields the next value
    pub fn next(&mut self, vm: &VM) -> Option<Value> {
        self.index += 1;
        let pk = (self.index - 1).to_string();
        self.value.get_property(vm, pk.into())
    }
}

/// The array constructor
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array-constructor
pub fn array_constructor(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array", ctx.vm)
}

/// This function implements Array.prototype.push
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.push
pub fn push(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.unwrap();

    let mut this = this_cell.borrow_mut(ctx.vm);
    let this_arr = match this.as_exotic_object_mut() {
        Some(ExoticObject::Array(a)) => a,
        _ => {
            return Err(
                error::create_error("Array.prototype.push called on non-array", ctx.vm).into(),
            )
        }
    };

    for value in ctx.args.iter().rev() {
        this_arr.elements.push(value.clone());
    }

    Ok((this_arr.elements.len() as f64).into())
}

/// This function implements Array.prototype.concat
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.concat
pub fn concat(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let this = this_cell.borrow(ctx.vm);
    let this_arr = match this.as_exotic_object() {
        Some(ExoticObject::Array(a)) => a,
        _ => {
            return Err(
                error::create_error("Array.prototype.concat called on non-array", ctx.vm).into(),
            );
        }
    };

    let mut arr = this_arr.clone();
    for arg in ctx.arguments() {
        arr.elements.push(arg.clone());
    }

    Ok(ctx.vm.register_array(arr).into())
}

/// This function implements Array.prototype.map
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.map
pub fn map(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let this_ref = this_cell.borrow(ctx.vm);
    let this_arr = match this_ref.as_exotic_object() {
        Some(ExoticObject::Array(a)) => a,
        _ => {
            return Err(
                error::create_error("Array.prototype.map called on non-array", ctx.vm).into(),
            );
        }
    };

    let cb = ctx.args.first().unwrap();

    let mut arr = Vec::new();

    for (idx, value) in this_arr.elements.iter().enumerate() {
        let idx = Value::from(idx as f64);
        let value = value.clone();

        arr.push(Value::call(cb, vec![value, idx], ctx.vm)?);
    }

    Ok(ctx.vm.register_array(Array::new(arr)).into())
}

/// This function implements Array.prototype.every
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.every
pub fn every(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow(ctx.vm);
    let this_arr = match this_ref.as_exotic_object() {
        Some(ExoticObject::Array(a)) => a,
        _ => {
            return Err(
                error::create_error("Array.prototype.every called on non-array", ctx.vm).into(),
            )
        }
    };

    let cb = ctx.args.first().unwrap();

    for value in &this_arr.elements {
        let value = cb.call(vec![value.clone()], ctx.vm)?;
        let is_truthy = value.is_truthy();

        if !is_truthy {
            return Ok(false.into());
        }
    }

    Ok(true.into())
}

/// This function implements Array.prototype.fill
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.fill
pub fn fill(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow_mut(ctx.vm);
    let this_arr = match this_ref.as_exotic_object_mut() {
        Some(ExoticObject::Array(a)) => a,
        _ => {
            return Err(
                error::create_error("Array.prototype.fill called on non-array", ctx.vm).into(),
            )
        }
    };

    let length = this_arr.elements.len();

    let mut args = ctx.arguments();
    let value = Value::unwrap_or_undefined(args.next().cloned(), ctx.vm);

    let start = args
        .next()
        .map(|c| c.as_number(ctx.vm) as usize)
        .map(|c| c.max(length))
        .unwrap_or(0);

    let end = args
        .next()
        .map(|c| c.as_number(ctx.vm) as usize)
        .map(|c| c.min(length))
        .unwrap_or_else(|| this_arr.elements.len());

    for idx in start..end {
        this_arr.elements[idx] = value.clone();
    }

    Ok(Handle::clone(this_cell).into())
}

/// This function implements Array.prototype.filter
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.filter
pub fn filter(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.filter", ctx.vm)
}

/// This function implements Array.prototype.find
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.find
pub fn find(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.find", ctx.vm)
}

/// This function implements Array.prototype.find
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.findIndex
pub fn find_index(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.findIndex", ctx.vm)
}

/// This function implements Array.prototype.flat
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.flat
pub fn flat(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.flat", ctx.vm)
}

/// This function implements Array.prototype.forEach
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.forEach
pub fn for_each(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.forEach", ctx.vm)
}

/// This function implements Array.from
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.from
pub fn from(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.from", ctx.vm)
}

/// This function implements Array.prototype.includes
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.includes
pub fn includes(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow(ctx.vm);
    let this_arr = match this_ref.as_exotic_object() {
        Some(ExoticObject::Array(a)) => a,
        _ => {
            return Err(
                error::create_error("Array.prototype.includes called on non-array", ctx.vm).into(),
            )
        }
    };

    let mut args = ctx.arguments();
    let search_element = Value::unwrap_or_undefined(args.next().cloned(), ctx.vm);
    let from_index = args
        .next()
        .map(|c| c.as_number(ctx.vm) as usize)
        .unwrap_or(0);

    let found = this_arr
        .elements
        .iter()
        .skip(from_index)
        .any(|c| c.strict_equal(&search_element));

    Ok(found.into())
}

/// This function implements Array.prototype.indexOf
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.indexOf
pub fn index_of(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this_ref.as_exotic_object_mut() {
        Some(ExoticObject::Array(a)) => a,
        _ => {
            return Err(
                error::create_error("Array.prototype.indexOf called on non-array", ctx.vm).into(),
            )
        }
    };

    let mut args = ctx.arguments();
    let search_element = Value::unwrap_or_undefined(args.next().cloned(), ctx.vm);
    let from_index = args
        .next()
        .map(|c| c.as_number(ctx.vm) as usize)
        .map(|c| c as usize)
        .unwrap_or(0);

    let index = this_arr
        .elements
        .iter()
        .skip(from_index)
        .position(|c| c.strict_equal(&search_element))
        .map(|v| v as f64)
        .unwrap_or(-1f64);

    Ok(index.into())
}

/// This function implements Array.isArray
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.isArray
pub fn is_array(ctx: CallContext) -> NativeFunctionCallbackResult {
    let mut arguments = ctx.arguments();
    let value_cell = arguments
        .next()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));
    let value = unsafe { value_cell.borrow_unbounded() };
    Ok(ctx
        .vm
        .create_js_value(value.as_object().and_then(Object::as_array).is_some())
        .into_handle(ctx.vm))
}

/// This function implements Array.prototype.join
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.join
pub fn join(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.as_ref();
    let this_ref = this_cell.map(|x| unsafe { x.borrow_unbounded() });
    let (len, mut this) = iterable_from_value(
        ctx.vm,
        this_cell,
        this_ref.as_ref().map(|x| &***x),
        "Array.prototype.join called on null value",
    )?;

    let separator = ctx.arguments().next().cloned();

    let sep = if let Some(separator_cell) = separator {
        Cow::Owned(
            unsafe { separator_cell.borrow_unbounded() }
                .to_string()
                .to_string(),
        )
    } else {
        Cow::Borrowed(",")
    };

    let mut r = String::new();

    while this.index < len {
        if this.index > 0 {
            r.push_str(&sep);
        }

        let element_cell = this
            .next(ctx.vm)
            .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));

        let is_nullish = unsafe { element_cell.borrow_unbounded() }.is_nullish();

        if !is_nullish {
            let element = abstractions::conversions::to_string(ctx.vm, Some(&element_cell))?;
            let element_ref = unsafe { element.borrow_unbounded() };
            // TODO: use as_string and throw if none
            r.push_str(&element_ref.to_string());
        }
    }

    Ok(ctx.vm.create_js_value(r).into_handle(ctx.vm))
}

/// This function implements Array.prototype.lastIndexOf
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.lastIndexOf
pub fn last_index_of(ctx: CallContext) -> NativeFunctionCallbackResult {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this_ref.as_exotic_object_mut() {
        Some(ExoticObject::Array(a)) => a,
        _ => {
            return Err(
                error::create_error("Array.prototype.indexOf called on non-array", ctx.vm).into(),
            )
        }
    };

    let len = this_arr.elements.len();

    let mut args = ctx.arguments();
    let search_element_cell = args
        .next()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));

    let search_element = unsafe { search_element_cell.borrow_unbounded() };
    let from_index = args
        .next()
        .map(|c| unsafe { c.borrow_unbounded() }.as_number())
        .map(|c| c as usize)
        .unwrap_or(len - 1);

    let skip = len - from_index - 1;

    let index = this_arr
        .elements
        .iter()
        .rev()
        .skip(skip)
        .position(|c| unsafe { c.borrow_unbounded() }.strict_equal(&search_element))
        .map(|c| len - c - skip - 1)
        .map(|c| c as f64)
        .unwrap_or(-1f64);

    Ok(ctx.vm.create_js_value(index).into_handle(ctx.vm))
}

/// This function implements Array.of
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.of
pub fn of(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.of", ctx.vm)
}

/// This function implements Array.prototype.pop
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.pop
pub fn pop(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.pop", ctx.vm)
}

/// This function implements Array.prototype.reduce
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduce
pub fn reduce(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.reduce", ctx.vm)
}

/// This function implements Array.prototype.reduceRight
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduceRight
pub fn reduce_right(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.reduceRight", ctx.vm)
}

/// This function implements Array.prototype.reverse
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reverse
pub fn reverse(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.reverse", ctx.vm)
}

/// This function implements Array.prototype.shift
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.shift
pub fn shift(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.shift", ctx.vm)
}

/// This function implements Array.prototype.slice
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.slice
pub fn slice(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.slice", ctx.vm)
}

/// This function implements Array.prototype.some
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.some
pub fn some(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.some", ctx.vm)
}

/// This function implements Array.prototype.sort
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.sort
pub fn sort(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.sort", ctx.vm)
}

/// This function implements Array.prototype.splice
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.splice
pub fn splice(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.splice", ctx.vm)
}

/// This function implements Array.prototype.unshift
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.unshift
pub fn unshift(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Array.prototype.unshift", ctx.vm)
}
