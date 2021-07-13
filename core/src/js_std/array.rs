use std::{borrow::Cow, str::Chars};

use crate::{
    gc::Handle,
    vm::{
        abstractions,
        value::{
            array::Array,
            function::{CallContext, CallResult},
            object::Object,
            Value, ValueKind,
        },
        VM,
    },
};

use super::error::{self, MaybeRc};

/// The array constructor
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array-constructor
pub fn array_constructor(_args: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.push
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.push
pub fn push(value: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = value.receiver.unwrap();

    let mut this = this_cell.borrow_mut();
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

    Ok(CallResult::Ready(
        value
            .vm
            .create_js_value(this_arr.elements.len() as f64)
            .into_handle(value.vm),
    ))
}

/// This function implements Array.prototype.concat
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.concat
pub fn concat(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this = this_cell.borrow_mut();
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

    Ok(CallResult::Ready(Value::from(arr).into_handle(ctx.vm)))
}

struct Map {
    pub result: Option<Vec<Handle<Value>>>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            result: Some(Vec::new()),
        }
    }
}

/// This function implements Array.prototype.map
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.map
pub fn map(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow_mut();
    let this_arr = match this_ref.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                MaybeRc::Owned("Array.prototype.map called on non-array"),
                ctx.vm,
            ))
        }
    };

    let state = ctx.state.get_or_insert_as(Map::new).unwrap();

    if let Some(response) = &ctx.function_call_response {
        let arr = state.result.as_mut().unwrap();
        arr.push(Handle::clone(response));
    }

    let length = state.result.as_ref().unwrap().len();

    if length == this_arr.elements.len() {
        let arr = state.result.take().unwrap();
        return Ok(CallResult::Ready(
            ctx.vm.create_array(Array::new(arr)).into_handle(ctx.vm),
        ));
    }

    let element = this_arr.elements.get(length).cloned().unwrap();

    let cb = ctx.args.first().unwrap();

    Ok(CallResult::UserFunction(
        Handle::clone(cb),
        vec![
            element,
            ctx.vm.create_js_value(length as f64).into_handle(ctx.vm),
            Handle::clone(this_cell),
        ],
    ))
}

struct Every {
    index: usize,
}

impl Every {
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

/// This function implements Array.prototype.every
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.every
pub fn every(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow_mut();
    let this_arr = match this_ref.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => {
            return Err(error::create_error(
                MaybeRc::Owned("Array.prototype.every called on non-array"),
                ctx.vm,
            ))
        }
    };

    let state = ctx.state.get_or_insert_as(Every::new).unwrap();

    if let Some(response_cell) = ctx.function_call_response {
        let response = response_cell.borrow().is_truthy();
        if !response {
            return Ok(CallResult::Ready(
                ctx.vm.create_js_value(false).into_handle(ctx.vm),
            ));
        }

        state.index += 1;
    }

    if state.index == this_arr.elements.len() {
        return Ok(CallResult::Ready(
            ctx.vm.create_js_value(true).into_handle(ctx.vm),
        ));
    }

    let cb = ctx.args.first().unwrap();

    let element = this_arr.elements.get(state.index).cloned().unwrap();

    Ok(CallResult::UserFunction(
        Handle::clone(cb),
        vec![
            element,
            ctx.vm
                .create_js_value(state.index as f64)
                .into_handle(ctx.vm),
            Handle::clone(this_cell),
        ],
    ))
}

/// This function implements Array.prototype.fill
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.fill
pub fn fill(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow_mut();
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
        .map(|c| c.borrow().as_number() as usize)
        .map(|c| c.max(length))
        .unwrap_or(0);
    let end = args
        .next()
        .map(|c| c.borrow().as_number() as usize)
        .map(|c| c.min(length))
        .unwrap_or_else(|| this_arr.elements.len());

    for idx in start..end {
        this_arr.elements[idx] = Handle::clone(&value);
    }

    Ok(CallResult::Ready(Handle::clone(this_cell)))
}

/// This function implements Array.prototype.filter
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.filter
pub fn filter(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.find
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.find
pub fn find(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.find
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.findIndex
pub fn find_index(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.flat
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.flat
pub fn flat(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.forEach
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.forEach
pub fn for_each(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.from
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.from
pub fn from(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.includes
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.includes
pub fn includes(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow_mut();
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
    let search_element = search_element_cell.borrow();
    let from_index = args
        .next()
        .map(|c| c.borrow().as_number())
        .map(|c| c as usize)
        .unwrap_or(0);

    let found = this_arr
        .elements
        .iter()
        .skip(from_index)
        .any(|cell| cell.borrow().strict_equal(&search_element));

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(found).into_handle(ctx.vm),
    ))
}

/// This function implements Array.prototype.indexOf
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.indexOf
pub fn index_of(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow_mut();
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
    let search_element = search_element_cell.borrow();
    let from_index = args
        .next()
        .map(|c| c.borrow().as_number())
        .map(|c| c as usize)
        .unwrap_or(0);

    let index = this_arr
        .elements
        .iter()
        .skip(from_index)
        .position(|cell| cell.borrow().strict_equal(&search_element))
        .map(|v| v as f64)
        .unwrap_or(-1f64);

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(index).into_handle(ctx.vm),
    ))
}

/// This function implements Array.isArray
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.isArray
pub fn is_array(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let mut arguments = ctx.arguments();
    let value_cell = arguments
        .next()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));
    let value = value_cell.borrow();
    Ok(CallResult::Ready(
        ctx.vm
            .create_js_value(value.as_object().and_then(Object::as_array).is_some())
            .into_handle(ctx.vm),
    ))
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

/// State for a call to Array.prototype.join
pub struct Join {
    dest: Option<String>,
    idx: usize,
}

impl Join {
    /// Creates new Join state
    pub fn new() -> Self {
        Self {
            dest: Some(String::new()),
            idx: 0,
        }
    }
}

/// This function implements Array.prototype.join
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.join
pub fn join(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();

    let len = abstractions::object::length_of_array_like(ctx.vm, this_cell)? as usize;
    let this_ref = this_cell.borrow_mut();

    let mut o = ArrayLikeIterable::from_value(&this_ref, this_cell);

    let separator = ctx.arguments().next().cloned();

    let sep = if let Some(separator_cell) = separator {
        Cow::Owned(separator_cell.borrow().to_string().to_string())
    } else {
        Cow::Borrowed(",")
    };

    let mut r = String::new();

    if let Some(response_cell) = &ctx.function_call_response {
        let response = response_cell.borrow();
        let response_string = response.as_string().ok_or_else(|| {
            error::create_error("Cannot convert to primitive value".into(), ctx.vm)
        })?;

        let state = ctx.state.get_or_insert_as(Join::new).unwrap();
        let dest = state.dest.as_mut().expect("cannot fail");

        r.push_str(dest);
        r.push_str(response_string);

        state.idx += 1;
        o.index = state.idx;

        if state.idx >= len {
            return Ok(CallResult::Ready(
                ctx.vm.create_js_value(r).into_handle(ctx.vm),
            ));
        }
    }

    while o.index < len {
        if o.index > 0 {
            r += &sep;
        }

        // TODO: unwrap_or_else is unnecessary here. to_string operation takes Option<&Value>
        let element_cell = o
            .next(ctx.vm)
            .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));

        let element = element_cell.borrow();

        if !element.is_nullish() {
            let next = match abstractions::conversions::to_string(ctx.vm, Some(&element_cell))? {
                CallResult::Ready(r) => r,
                CallResult::UserFunction(func, args) => {
                    let state = ctx.state.get_or_insert_as(Join::new).unwrap();
                    let dest = state.dest.as_mut().unwrap();
                    *dest = r;
                    return Ok(CallResult::UserFunction(func, args));
                }
            };

            let next_ref = next.borrow();
            let next_string = next_ref.as_string().ok_or_else(|| {
                error::create_error("Cannot convert to primitive value".into(), ctx.vm)
            })?;

            r += next_string;
        }
    }

    Ok(CallResult::Ready(
        ctx.vm
            .create_js_value(String::from(&*r))
            .into_handle(ctx.vm),
    ))
}

/// This function implements Array.prototype.lastIndexOf
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.lastIndexOf
pub fn last_index_of(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx.receiver.as_ref().unwrap();
    let mut this_ref = this_cell.borrow_mut();
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

    let search_element = search_element_cell.borrow();
    let from_index = args
        .next()
        .map(|c| c.borrow().as_number())
        .map(|c| c as usize)
        .unwrap_or(len - 1);

    let skip = len - from_index - 1;

    let index = this_arr
        .elements
        .iter()
        .rev()
        .skip(skip)
        .position(|c| c.borrow().strict_equal(&search_element))
        .map(|c| len - c - skip - 1)
        .map(|c| c as f64)
        .unwrap_or(-1f64);

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(index).into_handle(ctx.vm),
    ))
}

/// This function implements Array.of
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.of
pub fn of(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.pop
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.pop
pub fn pop(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.reduce
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduce
pub fn reduce(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.reduceRight
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduceRight
pub fn reduce_right(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.reverse
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reverse
pub fn reverse(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.shift
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.shift
pub fn shift(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.slice
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.slice
pub fn slice(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.some
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.some
pub fn some(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.sort
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.sort
pub fn sort(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.splice
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.splice
pub fn splice(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// This function implements Array.prototype.unshift
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.unshift
pub fn unshift(_ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}
