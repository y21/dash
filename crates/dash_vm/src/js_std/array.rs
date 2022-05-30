use crate::local::LocalScope;
use crate::throw;
use crate::value::array::Array;
use crate::value::array::ArrayIterator;
use crate::value::function::native::CallContext;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::ops::equality::ValueEquality;
use crate::value::Value;
use crate::value::ValueContext;

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

pub fn values(cx: CallContext) -> Result<Value, Value> {
    let iter = ArrayIterator::new(cx.scope, cx.this)?;
    Ok(cx.scope.register(iter).into())
}

pub fn at(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)? as i64;
    let mut index = cx.args.first().unwrap_or_undefined().to_integer_or_infinity(cx.scope)? as i64;

    if index < 0 {
        index = len + index;
    }

    if index < 0 || index >= len {
        return Ok(Value::undefined());
    }

    let index = index.to_string();
    this.get_property(cx.scope, index.as_str().into())
}

pub fn concat(cx: CallContext) -> Result<Value, Value> {
    let _this = Value::Object(cx.this.to_object(cx.scope)?);
    let mut array = Vec::new();
    // TODO: add elements from `this` to `array`

    for arg in &cx.args {
        let len = arg.length_of_array_like(cx.scope)?;
        for i in 0..len {
            let i = i.to_string();
            let element = arg.get_property(cx.scope, i.as_str().into())?;
            array.push(element);
        }
    }

    let array = Array::from_vec(cx.scope, array);

    Ok(cx.scope.register(array).into())
}

pub fn entries(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, "Not implemented")
}

pub fn keys(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, "Not implemented")
}

pub fn every(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv, Value::Number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;
        if !test {
            return Ok(false.into());
        }
    }

    Ok(true.into())
}

pub fn fill(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let value = cx.args.first().unwrap_or_undefined();

    for i in 0..len {
        let pk = i.to_string();
        this.set_property(cx.scope, pk.into(), value.clone())?;
    }

    Ok(this)
}

pub fn filter(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();
    let mut values = Vec::new();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv.clone(), Value::Number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;

        if test {
            cx.scope.add_value(pkv.clone());
            values.push(pkv);
        }
    }

    let values = Array::from_vec(cx.scope, values);

    Ok(cx.scope.register(values).into())
}

pub fn find(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv.clone(), Value::Number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;

        if test {
            return Ok(pkv);
        }
    }

    Ok(Value::undefined())
}

pub fn find_index(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv, Value::Number(k as f64)];
        let test = callback.apply(cx.scope, Value::undefined(), args)?.to_boolean()?;

        if test {
            return Ok(Value::Number(k as f64));
        }
    }

    Ok(Value::Number(-1.0))
}

pub fn flat(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, "Not implemented")
}

pub fn for_each(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv, Value::Number(k as f64)];
        callback.apply(cx.scope, Value::undefined(), args)?;
    }

    Ok(Value::undefined())
}

pub fn includes(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let search_element = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        if pkv.strict_eq(&search_element, cx.scope)?.is_truthy() {
            return Ok(true.into());
        }
    }

    Ok(false.into())
}

pub fn index_of(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let search_element = cx.args.first().unwrap_or_undefined();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        if pkv.strict_eq(&search_element, cx.scope)?.is_truthy() {
            return Ok(Value::Number(k as f64));
        }
    }

    Ok(Value::Number(-1.0))
}

pub fn map(cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;
    let callback = cx.args.first().unwrap_or_undefined();
    let mut values = Vec::new();

    for k in 0..len {
        let pk = k.to_string();
        let pkv = this.get_property(cx.scope, pk.as_str().into())?;
        let args = vec![pkv.clone(), Value::Number(k as f64)];
        let value = callback.apply(cx.scope, Value::undefined(), args)?;

        cx.scope.add_value(value.clone());
        values.push(value);
    }

    let values = Array::from_vec(cx.scope, values);

    Ok(cx.scope.register(values).into())
}

pub fn pop(mut cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    if len == 0 {
        return Ok(Value::undefined());
    }

    let new_len = len - 1;
    let value = this.delete_property(&mut cx.scope, new_len.to_string().into())?;
    this.set_property(&mut cx.scope, "length".into(), Value::Number(new_len as f64))?;

    Ok(value)
}

pub fn push(mut cx: CallContext) -> Result<Value, Value> {
    let this = Value::Object(cx.this.to_object(cx.scope)?);
    let len = this.length_of_array_like(cx.scope)?;

    let mut last = Value::undefined();

    if cx.args.is_empty() {
        this.set_property(&mut cx.scope, len.to_string().into(), Value::undefined())?;
    }

    for (idx, arg) in cx.args.into_iter().enumerate() {
        let pk = (idx + len).to_string();
        last = arg.clone();
        this.set_property(&mut cx.scope, pk.into(), arg)?;
    }

    Ok(last)
}
