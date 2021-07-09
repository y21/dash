use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::vm::value::{
    function::{CallContext, CallResult},
    object::{Object, Weak},
    weak::WeakSet,
    HashWeak, Value,
};

/// The WeakSet constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-weakset-constructor
pub fn weakset_constructor(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let elements_cell = ctx.args.get(0);
    let elements = elements_cell.map(|c| c.borrow());
    let arr = elements
        .as_deref()
        .and_then(Value::as_object)
        .and_then(Object::as_array)
        .unwrap();

    let mut set = HashSet::new();

    for arg in &arr.elements {
        let arg_weak = Rc::downgrade(arg);
        set.insert(HashWeak(arg_weak));
    }

    let ws = WeakSet::<RefCell<Value>>::from(set);
    Ok(CallResult::Ready(ctx.vm.create_js_value(ws).into()))
}

/// Implements WeakSet.prototype.has
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.has
pub fn has(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value_cell = ctx.args.get(0).unwrap();

    let this_ref = ctx.receiver.as_ref().map(|c| c.borrow());
    let this = this_ref
        .as_deref()
        .and_then(Value::as_object)
        .and_then(Object::as_weak)
        .and_then(Weak::as_set)
        .unwrap();

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(this.has(value_cell)).into(),
    ))
}

/// Implements WeakSet.prototype.add
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.add
pub fn add(mut ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value_cell = ctx.args.get(0).unwrap();

    let this = ctx.receiver.as_mut().unwrap();
    let mut this_ref = this.borrow_mut();
    let this_set = this_ref
        .as_object_mut()
        .and_then(Object::as_weak_mut)
        .and_then(Weak::as_set_mut)
        .unwrap();

    this_set.add(value_cell);

    Ok(CallResult::Ready(Rc::clone(&this)))
}

/// Implements WeakSet.prototype.delete
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.delete
pub fn delete(mut ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value_cell = ctx.args.get(0).unwrap();

    let this = ctx.receiver.as_mut().unwrap();
    let mut this_ref = this.borrow_mut();
    let this_set = this_ref
        .as_object_mut()
        .and_then(Object::as_weak_mut)
        .and_then(Weak::as_set_mut)
        .unwrap();

    let found = this_set.delete(value_cell);

    Ok(CallResult::Ready(ctx.vm.create_js_value(found).into()))
}
