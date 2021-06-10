use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::vm::value::{array::Array, function::CallContext, Value, ValueKind};

pub fn object_constructor(_args: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    Ok(Value::new(ValueKind::Undefined).into())
}

pub fn define_property(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let mut arguments = ctx.arguments();

    let obj_cell = arguments.next().unwrap();
    let mut obj = obj_cell.borrow_mut();
    let prop_cell = arguments.next().unwrap();
    let prop = prop_cell.borrow();
    let prop_str = prop.to_string();
    let descriptor_cell = arguments.next().unwrap();

    let value = Value::get_property(descriptor_cell, "value", None).unwrap();
    obj.set_property(&*prop_str, value);

    Ok(Rc::clone(&obj_cell))
}

pub fn get_own_property_names(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let obj_cell = ctx.args.first().unwrap();
    let obj = obj_cell.borrow();

    let mut keys = Vec::with_capacity(obj.fields.len());
    for key in obj.fields.keys() {
        let key: &str = &*key;
        keys.push(ctx.vm.create_js_value(String::from(key)).into());
    }

    Ok(ctx.vm.create_js_value(Array::new(keys)).into())
}

pub fn get_prototype_of(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let obj_cell = ctx.args.first().unwrap();
    let obj = obj_cell.borrow();
    Ok(obj
        .proto
        .as_ref()
        .and_then(Weak::upgrade)
        .unwrap_or_else(|| Value::new(ValueKind::Null).into()))
}
