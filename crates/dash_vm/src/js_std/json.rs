use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Value, ValueContext};
use crate::{json, throw};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, TypeError, "JSON is not a constructor")
}

pub fn parse(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    let bytes = value.res(cx.scope).as_bytes(); // TODO: thsi is probably going to be a borrowck issue
    let parse = match json::parser::Parser::new(bytes, cx.scope).parse() {
        Ok(v) => v,
        Err(e) => {
            throw!(cx.scope, SyntaxError, "{}", e.to_string())
        }
    };
    Ok(parse)
}
