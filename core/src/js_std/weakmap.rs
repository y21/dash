use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::vm::value::{
    function::{CallContext, CallResult},
    object::{Object, Weak},
    weak::WeakMap,
    HashWeak, Value, ValueKind,
};

/// The WeakMap constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-weakmap-constructor
pub fn weakmap_constructor(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let elements_cell = ctx.args.get(0);
    let elements = elements_cell.map(|c| c.borrow());
    let arr = elements
        .as_deref()
        .and_then(Value::as_object)
        .and_then(Object::as_array)
        .unwrap();

    let mut map = HashMap::new();

    for arg in &arr.elements {
        let arg_ref = arg.borrow();
        let arg = arg_ref.as_object().and_then(|x| x.as_array()).unwrap();
        let mut elements_iter = arg.elements.iter();

        let key = elements_iter.next().unwrap();
        let value = elements_iter.next().unwrap();

        let weak_key = HashWeak(Rc::downgrade(key));

        map.insert(weak_key, Rc::clone(&value));
    }

    let wm = WeakMap::from(map);
    Ok(CallResult::Ready(ctx.vm.create_js_value(wm).into()))
}

/// Implements WeakMap.prototype.has
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.has
pub fn has(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value_cell = ctx.args.get(0).unwrap();

    let this_ref = ctx.receiver.as_ref().map(|c| c.borrow());
    let this = this_ref
        .as_deref()
        .and_then(Value::as_object)
        .and_then(Object::as_weak)
        .and_then(Weak::as_map)
        .unwrap();

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(this.has_rc_key(value_cell)).into(),
    ))
}

/// Implements WeakMap.prototype.get
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.get
pub fn get(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value_cell = ctx.args.get(0).unwrap();

    let this_ref = ctx.receiver.as_ref().map(|c| c.borrow());
    let this = this_ref
        .as_deref()
        .and_then(Value::as_object)
        .and_then(Object::as_weak)
        .and_then(Weak::as_map)
        .unwrap();

    Ok(CallResult::Ready(
        this.get_rc_key(value_cell)
            .cloned()
            .unwrap_or_else(|| Value::new(ValueKind::Undefined).into()),
    ))
}

/// Implements WeakMap.prototype.add
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.add
pub fn add(mut args: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let (key, value) = {
        let mut arguments = args.arguments();
        (
            Rc::downgrade(arguments.next().unwrap()),
            Rc::clone(arguments.next().unwrap()),
        )
    };

    let this = args.receiver.as_mut().unwrap();
    let mut this_ref = this.borrow_mut();
    let this_map = this_ref
        .as_object_mut()
        .and_then(Object::as_weak_mut)
        .and_then(Weak::as_map_mut)
        .unwrap();

    this_map.add(key, value);

    Ok(CallResult::Ready(Rc::clone(&this)))
}

/// Implements WeakMap.prototype.delete
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.delete
pub fn delete(mut ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value_cell = ctx.args.get(0).unwrap();

    let this = ctx.receiver.as_mut().unwrap();
    let mut this_ref = this.borrow_mut();
    let this_map = this_ref
        .as_object_mut()
        .and_then(Object::as_weak_mut)
        .and_then(Weak::as_map_mut)
        .unwrap();

    let found = this_map.delete_rc_key(value_cell);

    Ok(CallResult::Ready(ctx.vm.create_js_value(found).into()))
}
