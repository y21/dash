use crate::vm::local::LocalScope;
use crate::vm::value::array::Array;
use crate::vm::value::function::native::CallContext;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::Value;
use crate::vm::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let array = Array::new(cx.scope);
    Ok(cx.scope.gc_mut().register(array).into())
}

fn join_inner(sc: &mut LocalScope, array: Value, separator: &str) -> Result<Value, Value> {
    let length = array.length_of_array_like(sc)?;

    let mut result = String::new();

    for i in 0..length {
        if i > 0 {
            result.push_str(separator);
        }

        let i = i.to_string();
        let element = array.get_property(sc, i.as_str().into())?;
        let s = element.to_string(sc)?;
        result.push_str(&s);
    }

    Ok(Value::String(result.into()))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    join_inner(cx.scope, cx.this, ",")
}

pub fn join(cx: CallContext) -> Result<Value, Value> {
    let sep = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    join_inner(cx.scope, cx.this, &sep)
}
