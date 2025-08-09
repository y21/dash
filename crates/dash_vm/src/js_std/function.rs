use crate::localscope::LocalScope;
use crate::throw;
use crate::value::function::Function;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::object::{Object, This};
use crate::value::ops::conversions::ValueConversion;
use crate::value::propertykey::ToPropertyKey;
use crate::value::{Root, Typeof, Unpack, Value, ValueKind};

use super::receiver_t;

pub fn constructor(_: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    throw!(scope, Error, "Dynamic code compilation is currently not supported")
}

pub fn apply(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = if let Some(array) = cx.args.get(1).cloned() {
        if array.is_nullish() {
            vec![]
        } else {
            let mut target_args = vec![];
            for i in 0..array.length_of_array_like(scope)? {
                let sym = i.to_key(scope);

                let arg_i = array.get_property(sym, scope).root(scope)?;
                target_args.push(arg_i);
            }
            target_args
        }
    } else {
        vec![]
    };

    let target_callee = match cx.this.unpack() {
        ValueKind::Object(o) if matches!(o.type_of(scope), Typeof::Function) => o,
        _ => throw!(scope, TypeError, "Bound value must be a function"),
    };

    target_callee
        .apply(
            target_this.map_or(This::default(), This::bound),
            target_args.into(),
            scope,
        )
        .root(scope)
}

pub fn bind(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec()).unwrap_or_default();
    let target_callee = match cx.this.unpack() {
        ValueKind::Object(o) if matches!(o.type_of(scope), Typeof::Function) => o,
        _ => throw!(scope, TypeError, "Bound value must be a function"),
    };

    let bf = BoundFunction::new(scope, target_callee, target_this, target_args.into());
    Ok(Value::object(scope.register(bf)))
}

pub fn call(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec()).unwrap_or_default();
    let target_callee = match cx.this.unpack() {
        ValueKind::Object(o) if matches!(o.type_of(scope), Typeof::Function) => o,
        _ => throw!(scope, TypeError, "Bound value must be a function"),
    };

    target_callee
        .apply(
            target_this.map_or(This::default(), This::bound),
            target_args.into(),
            scope,
        )
        .root(scope)
}

pub fn to_string(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let this = receiver_t::<Function>(scope, &cx.this, "Function.prototype.toString")?;
    let name = format!(
        "function {}() {{ [native code] }}",
        this.name().map(|s| s.res(scope)).unwrap_or_default()
    );
    Ok(Value::string(scope.intern(name.as_ref()).into()))
}
