use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{
    array::Array,
    function::{CallContext, CallResult},
    object::Object,
    Value, ValueKind,
};

use super::error::{self, MaybeRc};

pub fn array_constructor(_args: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    Ok(CallResult::Ready(Value::new(ValueKind::Undefined).into()))
}

pub fn push(value: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
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
        this_arr.elements.push(Rc::clone(&value));
    }

    Ok(CallResult::Ready(
        value
            .vm
            .create_js_value(this_arr.elements.len() as f64)
            .into(),
    ))
}

pub fn concat(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
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
        arr.elements.push(Rc::clone(arg));
    }

    Ok(CallResult::Ready(Value::from(arr).into()))
}

struct Map {
    pub result: Option<Vec<Rc<RefCell<Value>>>>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            result: Some(Vec::new()),
        }
    }
}

pub fn map(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
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
        arr.push(Rc::clone(response));
    }

    let length = state.result.as_ref().unwrap().len();

    if length == this_arr.elements.len() {
        let arr = state.result.take().unwrap();
        return Ok(CallResult::Ready(
            ctx.vm.create_array(Array::new(arr)).into(),
        ));
    }

    let element = this_arr.elements.get(length).cloned().unwrap();

    let cb = ctx.args.first().unwrap();

    Ok(CallResult::UserFunction(
        Rc::clone(cb),
        vec![
            element,
            ctx.vm.create_js_value(length as f64).into(),
            Rc::clone(this_cell),
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

pub fn every(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
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
            return Ok(CallResult::Ready(ctx.vm.create_js_value(false).into()));
        }

        state.index += 1;
    }

    if state.index == this_arr.elements.len() {
        return Ok(CallResult::Ready(ctx.vm.create_js_value(true).into()));
    }

    let cb = ctx.args.first().unwrap();

    let element = this_arr.elements.get(state.index).cloned().unwrap();

    Ok(CallResult::UserFunction(
        Rc::clone(cb),
        vec![
            element,
            ctx.vm.create_js_value(state.index as f64).into(),
            Rc::clone(this_cell),
        ],
    ))
}

pub fn fill(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
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
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into());

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
        this_arr.elements[idx] = Rc::clone(&value);
    }

    Ok(CallResult::Ready(Rc::clone(this_cell)))
}

pub fn filter(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn find(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn find_index(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn flat(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn for_each(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn from(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn includes(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
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
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into());
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

    Ok(CallResult::Ready(ctx.vm.create_js_value(found).into()))
}

pub fn index_of(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
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
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into());
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

    Ok(CallResult::Ready(ctx.vm.create_js_value(index).into()))
}

pub fn is_array(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let mut arguments = ctx.arguments();
    let value_cell = arguments
        .next()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into());
    let value = value_cell.borrow();
    Ok(CallResult::Ready(
        ctx.vm
            .create_js_value(value.as_object().and_then(Object::as_array).is_some())
            .into(),
    ))
}

pub fn join(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn last_index_of(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
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
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into());

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

    Ok(CallResult::Ready(ctx.vm.create_js_value(index).into()))
}

pub fn of(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn pop(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn reduce(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn reduce_right(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn reverse(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn shift(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn slice(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn some(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn sort(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn splice(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}

pub fn unshift(_ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    todo!()
}
