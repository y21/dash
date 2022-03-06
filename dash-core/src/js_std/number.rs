use crate::vm::value::function::native::CallContext;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::Value;
use crate::vm::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.get(0).unwrap_or_undefined().to_number()?;
    Ok(Value::Number(value))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    let radix = cx
        .args
        .get(0)
        .map(|v| v.to_number())
        .transpose()?
        .map(|n| n as u8)
        .unwrap_or(10);

    let num = cx.this.to_number()? as u64;

    let re = match radix {
        2 => format!("{:b}", num),
        10 => num.to_string(),
        16 => format!("{:x}", num),
        _ => todo!(),
    };

    Ok(Value::String(re.into()))
}
