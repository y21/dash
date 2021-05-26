use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::vm::value::{function::CallContext, object::Object, weak::WeakSet, HashWeak, Value};

pub fn weakset_constructor(value: CallContext) -> Rc<RefCell<Value>> {
    let elements_cell = value.args.get(0);
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
    Value::from(ws).into()
}

pub fn has(value: CallContext) -> Rc<RefCell<Value>> {
    let value_cell = value.args.get(0).unwrap();

    let this_ref = value.receiver.as_ref().map(|c| c.borrow());
    let this = this_ref
        .as_deref()
        .and_then(Value::as_object)
        .and_then(Object::as_weakset)
        .unwrap();

    Value::from(this.has(value_cell)).into()
}

pub fn add(mut value: CallContext) -> Rc<RefCell<Value>> {
    let value_cell = value.args.get(0).unwrap();

    let this = value.receiver.as_mut().unwrap();
    let mut this_ref = this.borrow_mut();
    let this_set = this_ref
        .as_object_mut()
        .and_then(Object::as_weakset_mut)
        .unwrap();

    this_set.add(value_cell);

    this.clone()
}

pub fn delete(mut value: CallContext) -> Rc<RefCell<Value>> {
    let value_cell = value.args.get(0).unwrap();

    let this = value.receiver.as_mut().unwrap();
    let mut this_ref = this.borrow_mut();
    let this_set = this_ref
        .as_object_mut()
        .and_then(Object::as_weakset_mut)
        .unwrap();

    let found = this_set.delete(value_cell);

    Value::from(found).into()
}
