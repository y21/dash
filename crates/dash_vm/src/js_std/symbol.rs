use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::primitive::Symbol;
use crate::value::{Value, ValueContext};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let description = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    let symbol = Symbol::new(description);
    Ok(symbol.into())
}
