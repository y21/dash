use crate::throw;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::function::Function;
use crate::value::object::Object;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Root, Typeof, Unpack, Value, ValueContext, ValueKind};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Dynamic code compilation is currently not supported")
}

pub fn apply(cx: CallContext) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = if let Some(array) = cx.args.get(1).cloned() {
        if array.is_nullish() {
            vec![]
        } else {
            let mut target_args = vec![];
            for i in 0..array.length_of_array_like(cx.scope)? {
                let sym = cx.scope.intern_usize(i).into();

                let arg_i = array.get_property(cx.scope, sym).root(cx.scope)?;
                target_args.push(arg_i);
            }
            target_args
        }
    } else {
        vec![]
    };

    let target_callee = match cx.this.unpack() {
        ValueKind::Object(o) if matches!(o.type_of(&cx.scope), Typeof::Function) => o,
        _ => throw!(cx.scope, TypeError, "Bound value must be a function"),
    };

    target_callee
        .apply(cx.scope, target_this.unwrap_or_undefined(), target_args)
        .root(cx.scope)
}

pub fn bind(cx: CallContext) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec());
    let target_callee = match cx.this.unpack() {
        ValueKind::Object(o) if matches!(o.type_of(&cx.scope), Typeof::Function) => o,
        _ => throw!(cx.scope, TypeError, "Bound value must be a function"),
    };

    let bf = BoundFunction::new(cx.scope, target_callee, target_this, target_args);
    Ok(Value::object(cx.scope.register(bf)))
}

pub fn call(cx: CallContext) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec());
    let target_callee = match cx.this.unpack() {
        ValueKind::Object(o) if matches!(o.type_of(&cx.scope), Typeof::Function) => o,
        _ => throw!(cx.scope, TypeError, "Bound value must be a function"),
    };

    target_callee
        .apply(
            cx.scope,
            target_this.unwrap_or_undefined(),
            target_args.unwrap_or_default(),
        )
        .root(cx.scope)
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let Some(this) = this.downcast_ref::<Function>(&cx.scope) else {
        throw!(cx.scope, TypeError, "Incompatible receiver");
    };
    let name = format!(
        "function {}() {{ [native code] }}",
        this.name().map(|s| s.res(cx.scope)).unwrap_or_default()
    );
    Ok(Value::string(cx.scope.intern(name.as_ref()).into()))
}
