use std::{borrow::Cow, str::Chars};

use crate::{
    gc::Handle,
    vm::{
        abstractions,
        value::{array::Array, function::CallContext, object::Object, Value, ValueKind},
        VM,
    },
};

use super::error::{self, MaybeRc};

/// The array constructor
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array-constructor
pub fn array_constructor(_args: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.push
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.push
pub fn push(value: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = value.receiver.unwrap();

    let mut this = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                MaybeRc::Owned("Array.prototype.push called on non-array"),
                value.vm,
            ))
        }
    };

    for value in value.args.into_iter().rev() {
        this_arr.elements.push(Handle::clone(&value));
    }

    Ok(value
        .vm
        .create_js_value(this_arr.elements.len() as f64)
        .into_handle(value.vm))
}

/// This function implements Array.prototype.concat
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.concat
pub fn concat(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                MaybeRc::Owned("Array.prototype.concat called on non-array"),
                ctx.vm,
            ))
        }
    };

    let mut arr = this_arr.clone();
    for arg in ctx.arguments() {
        arr.elements.push(Handle::clone(arg));
    }

    Ok(Value::from(arr).into_handle(ctx.vm))
}

/// This function implements Array.prototype.map
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.map
pub fn map(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let this_ref = unsafe { this_cell.borrow_unbounded() };
    let this_arr = match this_ref.as_object() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                MaybeRc::Owned("Array.prototype.map called on non-array"),
                ctx.vm,
            ))
        }
    };

    let cb = ctx.args.first().unwrap();

    let mut arr = Vec::new();

    for (idx, value) in this_arr.elements.iter().enumerate() {
        let idx = ctx.vm.create_js_value(idx as f64).into_handle(ctx.vm);
        let value = Handle::clone(&value);

        arr.push(Value::call(cb, vec![value, idx], ctx.vm)?);
    }

    Ok(ctx.vm.create_js_value(Array::new(arr)).into_handle(ctx.vm))
}

/// This function implements Array.prototype.every
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.every
pub fn every(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this_ref.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                MaybeRc::Owned("Array.prototype.every called on non-array"),
                ctx.vm,
            ))
        }
    };

    let cb = ctx.args.first().unwrap();

    for value in &this_arr.elements {
        let value = Value::call(cb, vec![Handle::clone(value)], ctx.vm)?;
        let is_truthy = unsafe { value.borrow_unbounded() }.is_truthy();

        if !is_truthy {
            return Ok(ctx.vm.create_js_value(false).into_handle(ctx.vm));
        }
    }

    Ok(ctx.vm.create_js_value(true).into_handle(ctx.vm))
}

/// This function implements Array.prototype.fill
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.fill
pub fn fill(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this_ref.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                "Array.prototype.fill called on non-array".into(),
                ctx.vm,
            ))
        }
    };

    let length = this_arr.elements.len();

    let mut args = ctx.arguments();
    let value = args
        .next()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));

    let start = args
        .next()
        .map(|c| unsafe { c.borrow_unbounded() }.as_number() as usize)
        .map(|c| c.max(length))
        .unwrap_or(0);
    let end = args
        .next()
        .map(|c| unsafe { c.borrow_unbounded() }.as_number() as usize)
        .map(|c| c.min(length))
        .unwrap_or_else(|| this_arr.elements.len());

    for idx in start..end {
        this_arr.elements[idx] = Handle::clone(&value);
    }

    Ok(Handle::clone(this_cell))
}

/// This function implements Array.prototype.filter
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.filter
pub fn filter(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.find
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.find
pub fn find(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.find
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.findIndex
pub fn find_index(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.flat
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.flat
pub fn flat(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.forEach
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.forEach
pub fn for_each(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.from
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.from
pub fn from(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.includes
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.includes
pub fn includes(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this_ref.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                "Array.prototype.includes called on non-array".into(),
                ctx.vm,
            ))
        }
    };

    let mut args = ctx.arguments();
    let search_element_cell = args
        .next()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));
    let search_element = unsafe { search_element_cell.borrow_mut_unbounded() };
    let from_index = args
        .next()
        .map(|c| unsafe { c.borrow_unbounded() }.as_number())
        .map(|c| c as usize)
        .unwrap_or(0);

    let found = this_arr
        .elements
        .iter()
        .skip(from_index)
        .any(|c| unsafe { c.borrow_unbounded() }.strict_equal(&search_element));

    Ok(ctx.vm.create_js_value(found).into_handle(ctx.vm))
}

/// This function implements Array.prototype.indexOf
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.indexOf
pub fn index_of(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this_ref.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                "Array.prototype.indexOf called on non-array".into(),
                ctx.vm,
            ))
        }
    };

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
        .unwrap_or(0);

    let index = this_arr
        .elements
        .iter()
        .skip(from_index)
        .position(|cell| unsafe { cell.borrow_unbounded() }.strict_equal(&search_element))
        .map(|v| v as f64)
        .unwrap_or(-1f64);

    Ok(ctx.vm.create_js_value(index).into_handle(ctx.vm))
}

/// This function implements Array.isArray
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.isArray
pub fn is_array(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
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

/// An array-like value
pub enum ArrayLikeKind<'a> {
    /// Iterator over characters of a string
    String(Chars<'a>),
    /// Array of JS Values
    Array(&'a [Handle<Value>]),
    /// Javascript object
    Object(&'a Handle<Value>),
    /// No value
    Empty,
}

/// An array-like value that can be iterated over
pub struct ArrayLikeIterable<'a> {
    /// What kind of array
    pub kind: ArrayLikeKind<'a>,
    /// Current index
    pub index: usize,
}

impl<'a> ArrayLikeIterable<'a> {
    /// Creates a new array like iterable given an [ArrayLikeKind]
    pub fn new(kind: ArrayLikeKind<'a>) -> Self {
        Self { kind, index: 0 }
    }
    /// Creates a new array like iterable given a Value by detecting its kind
    pub fn from_value(value: &'a Value, value_cell: &'a Handle<Value>) -> Self {
        match value.as_object() {
            Some(Object::String(s)) => Self::new(ArrayLikeKind::String(s.chars())),
            Some(Object::Array(a)) => Self::new(ArrayLikeKind::Array(&a.elements)),
            Some(Object::Any(_)) => Self::new(ArrayLikeKind::Object(value_cell)),
            _ => Self::new(ArrayLikeKind::Empty),
        }
    }
    /// Yields the next value
    pub fn next<'b>(&mut self, vm: &'b mut VM) -> Option<Handle<Value>> {
        self.index += 1;
        match &mut self.kind {
            ArrayLikeKind::String(s) => s
                .next()
                .map(String::from)
                .map(Value::from)
                .map(|v| v.into_handle(vm)),
            ArrayLikeKind::Array(source) => source.get(self.index - 1).cloned(),
            ArrayLikeKind::Object(source_cell) => {
                Value::get_property(vm, source_cell, &(self.index - 1).to_string(), None)
            }
            ArrayLikeKind::Empty => None,
        }
    }
}

/// This function implements Array.prototype.join
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.join
pub fn join(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();

    let len = abstractions::object::length_of_array_like(ctx.vm, this_cell)? as usize;
    let this_ref = unsafe { this_cell.borrow_mut_unbounded() };

    let mut o = ArrayLikeIterable::from_value(&this_ref, this_cell);

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

    while o.index < len {
        if o.index > 0 {
            r.push_str(&sep);
        }

        let element_cell = o
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
pub fn last_index_of(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = unsafe { this_cell.borrow_mut_unbounded() };
    let this_arr = match this_ref.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                "Array.prototype.indexOf called on non-array".into(),
                ctx.vm,
            ))
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
pub fn of(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.pop
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.pop
pub fn pop(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.reduce
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduce
pub fn reduce(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.reduceRight
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduceRight
pub fn reduce_right(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.reverse
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reverse
pub fn reverse(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.shift
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.shift
pub fn shift(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.slice
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.slice
pub fn slice(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.some
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.some
pub fn some(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.sort
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.sort
pub fn sort(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.splice
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.splice
pub fn splice(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.unshift
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.unshift
pub fn unshift(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}
