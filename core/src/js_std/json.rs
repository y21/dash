use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::{
    js_std::error::{self, MaybeRc},
    json::parser::Parser,
    vm::value::{function::CallContext, object::Object, Value, ValueKind},
};

pub fn parse(value: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let source_cell = value
        .args
        .first()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into());

    let source = source_cell.borrow();
    let source_str = source.to_string();

    let parsed = Parser::new(source_str.as_bytes())
        .parse()
        .map_err(|e| error::create_error(MaybeRc::Owned(&e.to_string()), value.vm))?
        .into_js_value()
        .map_err(|e| error::create_error(MaybeRc::Owned(&e.to_string()), value.vm))?;

    Ok(parsed.into())
}

pub fn stringify(value: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let target = value.args.first().map(|c| c.borrow());

    let result = target
        .as_ref()
        .and_then(|c| c.to_json())
        .unwrap_or(Cow::Borrowed("undefined"));

    Ok(Value::from(Object::String(String::from(result))).into())
}
