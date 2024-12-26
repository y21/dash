use crate::value::error::{
    AggregateError, Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError,
};
use crate::value::function::native::CallContext;
use crate::value::object::NamedObject;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Root, Value, ValueContext};
use dash_middle::interner::sym;

macro_rules! define_other_error_constructors {
    ( $( $fun:ident $t:ident ),* ) => {
        $(
            pub fn $fun(mut cx: CallContext) -> Result<Value, Value> {
                let message = cx.args.first().unwrap_or_undefined().to_js_string(&mut cx.scope)?;
                let obj = if let Some(new_target) = cx.new_target {
                    NamedObject::instance_for_new_target(new_target, cx.scope)?
                } else {
                    $t::object(cx.scope)
                };
                let error = $t::new_with_js_string(cx.scope, obj, message);

                Ok(cx.scope.register(error).into())
            }
        )*
    };
}
define_other_error_constructors!(
    eval_error_constructor EvalError,
    range_error_constructor RangeError,
    reference_error_constructor ReferenceError,
    syntax_error_constructor SyntaxError,
    type_error_constructor TypeError,
    uri_error_constructor URIError,
    aggregate_error_constructor AggregateError
);

pub fn error_constructor(cx: CallContext) -> Result<Value, Value> {
    let message = cx.args.first().cloned().map(|v| v.to_js_string(cx.scope)).transpose()?;

    let new_target = cx.new_target.unwrap_or(cx.scope.statics.error_ctor);
    let err = Error::with_obj(
        NamedObject::instance_for_new_target(new_target, cx.scope)?,
        cx.scope,
        message.unwrap_or(sym::empty.into()),
    );

    Ok(cx.scope.register(err).into())
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    cx.this
        .get_property(cx.scope, sym::stack.into())
        .root(cx.scope)
        .and_then(|v| v.to_js_string(cx.scope).map(Value::string))
}
