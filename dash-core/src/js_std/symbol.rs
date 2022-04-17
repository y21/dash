use crate::vm::value::function::native::CallContext;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::primitive::Symbol;
use crate::vm::value::Value;
use crate::vm::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let description = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    let symbol = Symbol::new(description);
    Ok(symbol.into())
}
