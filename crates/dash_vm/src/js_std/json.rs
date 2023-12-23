use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Value, ValueContext};
use crate::{json, throw};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, TypeError, "JSON is not a constructor")
}

pub fn parse(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    let parse = match json::parser::Parser::new(value.as_bytes(), cx.scope).parse() {
        Ok(v) => v,
        Err(e) => {
            throw!(cx.scope, SyntaxError, "{}", e.to_string())
        }
    };
    Ok(parse)
}
