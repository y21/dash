use crate::value::error::{
    AggregateError, Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError,
};
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Root, Value, ValueContext};

macro_rules! define_other_error_constructors {
    ( $( $fun:ident $t:ident ),* ) => {
        $(
        pub fn $fun(mut cx: CallContext) -> Result<Value, Value> {
            let message = cx.args.first().unwrap_or_undefined().to_string(&mut cx.scope)?;
            let error = $t::new(&cx.scope, message);

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
    let message = cx.args.first().cloned().map(|v| v.to_string(cx.scope)).transpose()?;

    let err = Error::new(cx.scope, message.as_deref().unwrap_or_default());

    Ok(cx.scope.register(err).into())
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    cx.this
        .get_property(cx.scope, "stack".into())
        .root(cx.scope)
        .and_then(|v| v.to_string(cx.scope).map(Value::String))
}
