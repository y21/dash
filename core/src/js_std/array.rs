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

struct MapState {
    pub result: Option<Vec<Rc<RefCell<Value>>>>,
}

impl MapState {
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

    let state = ctx.state.get_or_insert_as(MapState::new).unwrap();

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
