use crate::localscope::LocalScope;
use crate::value::error::{
    AggregateError, Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError,
};
use crate::value::function::native::CallContext;
use crate::value::object::OrdObject;
use crate::value::ops::conversions::ValueConversion;
use crate::value::propertykey::ToPropertyKey;
use crate::value::{Root, Value, ValueContext};
use dash_middle::interner::sym;

macro_rules! define_other_error_constructors {
    ( $( $fun:ident $t:ident ),* ) => {
        $(
            pub fn $fun(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
                let message = cx.args.first().unwrap_or_undefined().to_js_string( scope)?;
                let obj = if let Some(new_target) = cx.new_target {
                    OrdObject::instance_for_new_target(new_target, scope)?
                } else {
                    $t::object(scope)
                };
                let error = $t::new_with_js_string(scope, obj, message);

                Ok(scope.register(error).into())
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

pub fn error_constructor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let message = cx.args.first().cloned().map(|v| v.to_js_string(scope)).transpose()?;

    let new_target = cx.new_target.unwrap_or(scope.statics.error_ctor);
    let err = Error::with_obj(
        OrdObject::instance_for_new_target(new_target, scope)?,
        scope,
        message.unwrap_or(sym::empty.into()),
    );

    Ok(scope.register(err).into())
}

pub fn to_string(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    cx.this
        .get_property(sym::stack.to_key(scope), scope)
        .root(scope)
        .and_then(|v| v.to_js_string(scope).map(Value::string))
}
