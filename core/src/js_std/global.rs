use crate::vm::value::{
    function::native::CallContext, ops::abstractions::conversions::ValueConversion, Value,
    ValueContext,
};

#[rustfmt::skip]
pub fn is_nan(cx: CallContext) -> Result<Value, Value> {
    // 1. Let num be ? ToNumber(number).
    let num = cx.args.first().unwrap_or_undefined().to_number()?;
    // 2. If num is NaN, return true.
    // 3. Otherwise, return false.
    Ok(Value::Boolean(num.is_nan()))
}
