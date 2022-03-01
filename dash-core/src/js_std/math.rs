use crate::vm::value::function::native::CallContext;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::Value;
use crate::vm::value::ValueContext;

pub fn floor(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number()?;

    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    if n.is_nan() || n.is_infinite() || n == 0f64 {
        return Ok(Value::Number(0f64));
    }

    Ok(Value::Number(n.floor()))
}
