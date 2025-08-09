use crate::localscope::LocalScope;
use crate::value::function::native::CallContext;
use crate::value::{ExceptionContext, Value};

pub mod array;
pub mod array_iterator;
pub mod arraybuffer;
pub mod boolean;
pub mod date;
pub mod error;
pub mod function;
pub mod generator;
pub mod global;
pub mod json;
pub mod map;
pub mod math;
pub mod number;
pub mod object;
pub mod promise;
pub mod regex;
pub mod set;
pub mod string;
pub mod symbol;
pub mod typedarray;
pub mod weakmap;
pub mod weakset;

pub fn receiver_t<'a, T: 'static>(
    sc: &mut LocalScope<'_>,
    value: &'a Value,
    what: &'static str,
) -> Result<&'a T, Value> {
    value
        .extract(sc)
        .or_type_err_args(sc, format_args!("{what} invoked on incompatible receiver"))
}

pub fn identity_this(cx: CallContext, _: &mut LocalScope<'_>) -> Result<Value, Value> {
    Ok(cx.this)
}
