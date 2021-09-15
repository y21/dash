use crate::{
    gc::Handle,
    js_std,
    vm::{
        abstractions,
        value::{function::CallContext, symbol::Symbol, Value, ValueKind},
    },
};

/// The Symbol constructor
pub fn symbol_constructor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let description = match ctx.args.first() {
        Some(handle) => Some(abstractions::conversions::to_string(ctx.vm, Some(handle))?),
        None => None,
    };

    let description_ref = description
        .as_ref()
        .map(|x| unsafe { x.borrow_unbounded() });

    let description_s = description_ref
        .as_ref()
        .and_then(|x| x.as_string())
        .map(ToOwned::to_owned);

    let symbol = Symbol(description_s.map(Into::into));

    Ok(ctx.vm.create_js_value(symbol).into_handle(ctx.vm))
}

/// Implements Symbol.for
pub fn symbol_for(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let description = abstractions::conversions::to_string(ctx.vm, ctx.args.first())?;

    let description_ref = unsafe { description.borrow_unbounded() };

    let description_s = description_ref.to_string();

    let symbol = match ctx.vm.symbols.get(description_s.as_ref()).cloned() {
        Some(value) => value,
        None => {
            let value = ctx
                .vm
                .create_js_value(Symbol(Some(description_s.to_owned().into())))
                .into_handle(ctx.vm);

            ctx.vm
                .symbols
                .insert(description_s.into_owned().into(), Handle::clone(&value));

            value
        }
    };

    Ok(symbol)
}

/// Implements Symbol.keyFor
pub fn symbol_key_for(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let param = ctx.args.first();
    let param_ref = param.map(|x| unsafe { x.borrow_unbounded() });

    let param_obj = param_ref
        .as_ref()
        .and_then(|x| x.as_object())
        .and_then(|x| x.as_symbol())
        .ok_or_else(|| js_std::error::create_error("Provided value is not a symbol", ctx.vm))?;

    let desc = param_obj.0.as_deref().unwrap_or("undefined");

    let is_shared = ctx
        .vm
        .symbols
        .get(desc)
        .zip(param)
        .map(|(x, y)| std::ptr::eq(x.as_ptr(), y.as_ptr()))
        .unwrap_or_default();

    if is_shared {
        Ok(ctx
            .vm
            .create_js_value(String::from(desc))
            .into_handle(ctx.vm))
    } else {
        Ok(Value::new(ValueKind::Undefined).into_handle(ctx.vm))
    }
}
