use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::vm::value::{
    function::CallContext,
    object::{Object, Weak},
    weak::WeakMap,
    HashWeak, Value, ValueKind,
};

pub fn weakmap_constructor(value: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let elements_cell = value.args.get(0);
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

        map.insert(weak_key, value.clone());
    }

    let wm = WeakMap::from(map);
    Ok(Value::from(wm).into())
}

pub fn has(value: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let value_cell = value.args.get(0).unwrap();

    let this_ref = value.receiver.as_ref().map(|c| c.borrow());
    let this = this_ref
        .as_deref()
        .and_then(Value::as_object)
        .and_then(Object::as_weak)
        .and_then(Weak::as_map)
        .unwrap();

    Ok(Value::from(this.has_rc_key(value_cell)).into())
}

pub fn get(value: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let value_cell = value.args.get(0).unwrap();

    let this_ref = value.receiver.as_ref().map(|c| c.borrow());
    let this = this_ref
        .as_deref()
        .and_then(Value::as_object)
        .and_then(Object::as_weak)
        .and_then(Weak::as_map)
        .unwrap();

    Ok(this
        .get_rc_key(value_cell)
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into()))
}

pub fn add(mut args: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
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

    Ok(this.clone())
}

pub fn delete(mut value: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let value_cell = value.args.get(0).unwrap();

    let this = value.receiver.as_mut().unwrap();
    let mut this_ref = this.borrow_mut();
    let this_map = this_ref
        .as_object_mut()
        .and_then(Object::as_weak_mut)
        .and_then(Weak::as_map_mut)
        .unwrap();

    let found = this_map.delete_rc_key(value_cell);

    Ok(Value::from(found).into())
}
