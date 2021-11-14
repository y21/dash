use std::borrow::Cow;

use crate::{
    js_std::error::{self},
    json::parser::Parser,
    vm::value::{
        function::{CallContext, NativeFunctionCallbackResult},
        object::Object,
        Value,
    },
};

/// Implements JSON.parse
///
/// https://tc39.es/ecma262/multipage/structured-data.html#sec-json.parse
pub fn parse(ctx: CallContext) -> NativeFunctionCallbackResult {
    let source = Value::unwrap_or_undefined(ctx.args.first().cloned(), ctx.vm);

    let source_str = source.to_string(ctx.vm);

    let value = Parser::new(source_str.as_bytes())
        .parse()
        .map_err(|e| error::create_error(e.to_string(), ctx.vm))?
        .into_js_value(ctx.vm)
        .map_err(|e| error::create_error(e.to_string(), ctx.vm))?;

    Ok(value)
}

/// Implements JSON.stringify
///
/// https://tc39.es/ecma262/multipage/structured-data.html#sec-json.stringify
pub fn stringify(ctx: CallContext) -> NativeFunctionCallbackResult {
    let result = ctx
        .args
        .first()
        .and_then(|c| c.to_json(ctx.vm))
        .unwrap_or(Cow::Borrowed("undefined"));

    Ok(ctx
        .vm
        .register_object(Object::from(result.into_owned()))
        .into())
}
