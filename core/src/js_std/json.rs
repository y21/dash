use std::borrow::Cow;

use crate::{
    gc::Handle,
    js_std::error::{self},
    json::parser::Parser,
    vm::value::{function::CallContext, Value, ValueKind},
};

/// Implements JSON.parse
///
/// https://tc39.es/ecma262/multipage/structured-data.html#sec-json.parse
pub fn parse(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let source_cell = ctx
        .args
        .first()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));

    let source = unsafe { source_cell.borrow_unbounded() };
    let source_str = source.to_string();

    let parsed = Parser::new(source_str.as_bytes())
        .parse()
        .map_err(|e| error::create_error(e.to_string(), ctx.vm))?
        .into_js_value(ctx.vm)
        .map_err(|e| error::create_error(e.to_string(), ctx.vm))?;

    Ok(parsed.into_handle(ctx.vm))
}

/// Implements JSON.stringify
///
/// https://tc39.es/ecma262/multipage/structured-data.html#sec-json.stringify
pub fn stringify(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let target = ctx.args.first().map(|c| unsafe { c.borrow_unbounded() });

    let result = target
        .as_ref()
        .and_then(|c| c.to_json())
        .unwrap_or(Cow::Borrowed("undefined"));

    Ok(ctx
        .vm
        .create_js_value(String::from(result))
        .into_handle(ctx.vm))
}
