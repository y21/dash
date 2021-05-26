use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{array::Array, function::CallContext, object::Object, Value, ValueKind};

pub fn define_property(value: CallContext) -> Rc<RefCell<Value>> {
    let mut arguments = value.arguments();

    let obj_cell = arguments.next().unwrap();
    let mut obj = obj_cell.borrow_mut();
    let prop_cell = arguments.next().unwrap();
    let prop = prop_cell.borrow();
    let prop_str = prop.as_string_lossy().unwrap();
    let descriptor_cell = arguments.next().unwrap();

    let value = Value::get_property(descriptor_cell, "value").unwrap();
    obj.set_property(&*prop_str, value);

    obj_cell.clone()
}

pub fn get_own_property_names(value: CallContext) -> Rc<RefCell<Value>> {
    let obj_cell = value.args.first().unwrap();
    let obj = obj_cell.borrow();

    let mut keys = Vec::with_capacity(obj.fields.len());
    for key in obj.fields.keys() {
        let key: &str = &*key;
        keys.push(Value::from(String::from(key)).into());
    }

    Value::from(Array::new(keys)).into()
}
