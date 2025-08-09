use crate::localscope::LocalScope;
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Value, ValueContext};
use crate::{json, throw};

pub fn constructor(_: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    throw!(scope, TypeError, "JSON is not a constructor")
}

pub fn parse(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined().to_js_string(scope)?;
    let bytes = value.res(scope).as_bytes().to_owned();
    let parse = match json::parser::Parser::new(&bytes, scope).parse() {
        Ok(v) => v,
        Err(e) => {
            throw!(scope, SyntaxError, "{}", e.to_string())
        }
    };
    Ok(parse)
}
