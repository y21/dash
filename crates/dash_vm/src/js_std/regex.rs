use crate::localscope::LocalScope;
use crate::throw;
use crate::value::array::Array;
use crate::value::function::native::CallContext;
use crate::value::object::{OrdObject, PropertyValue};
use crate::value::ops::conversions::ValueConversion;
use crate::value::regex::{RegExp, RegExpInner};
use crate::value::{Value, ValueContext};
use dash_middle::interner::sym;
use dash_regex::{EvalSuccess, Flags};

use super::receiver_t;

pub fn constructor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let pattern = cx.args.first().unwrap_or_undefined().to_js_string(scope)?;
    let flags = match cx
        .args
        .get(1)
        .map(|v| v.to_js_string(scope))
        .transpose()?
        .map(|s| s.res(scope).parse::<Flags>())
    {
        Some(Ok(flags)) => flags,
        Some(Err(err)) => throw!(scope, SyntaxError, "Invalid RegExp flags: {:?}", err),
        None => Flags::empty(),
    };

    let nodes = match dash_regex::compile(pattern.res(scope), flags) {
        Ok(nodes) => nodes,
        Err(err) => throw!(scope, SyntaxError, "Regex parser error: {}", err),
    };

    let new_target = cx.new_target.unwrap_or(scope.statics.regexp_ctor);
    let regex = RegExp::with_obj(nodes, pattern, OrdObject::instance_for_new_target(new_target, scope)?);

    Ok(Value::object(scope.register(regex)))
}

pub fn test(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let text = cx.args.first().unwrap_or_undefined().to_js_string(scope)?;

    let regex = receiver_t::<RegExp>(scope, &cx.this, "RegExp.prototype.test")?;

    let RegExpInner { regex, last_index, .. } = match regex.inner() {
        Some(nodes) => nodes,
        None => throw!(scope, TypeError, "Receiver must be an initialized RegExp object"),
    };

    let text = text.res(scope);
    let is_global = regex.flags().contains(Flags::GLOBAL);

    if is_global && last_index.get() >= text.len() {
        last_index.set(0);
        return Ok(Value::boolean(false));
    }

    match regex.eval(&text[last_index.get()..]) {
        Ok(EvalSuccess { groups }) => {
            if is_global {
                last_index.set(last_index.get() + groups[0].unwrap().1 as usize);
            }
            Ok(Value::boolean(true))
        }
        Err(_) => {
            if is_global {
                last_index.set(0);
            }
            Ok(Value::boolean(false))
        }
    }
}

pub fn exec(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let text = cx.args.first().unwrap_or_undefined().to_js_string(scope)?;

    let regex = receiver_t::<RegExp>(scope, &cx.this, "RegExp.prototype.exec")?;

    let RegExpInner { regex, last_index, .. } = match regex.inner() {
        Some(nodes) => nodes,
        None => throw!(scope, TypeError, "Receiver must be an initialized RegExp object"),
    };

    let text = text.res(scope).to_owned();
    let is_global = regex.flags().contains(Flags::GLOBAL);

    if is_global && last_index.get() >= text.len() {
        last_index.set(0);
        return Ok(Value::null());
    }

    match regex.eval(&text[last_index.get()..]) {
        Ok(EvalSuccess { groups }) => {
            if is_global {
                last_index.set(last_index.get() + groups[0].unwrap().1 as usize);
            }

            let groups = groups
                .into_iter()
                .map(|group| {
                    let sub = match group {
                        Some((from, to, _)) => scope.intern(&text[from as usize..to as usize]).into(),
                        None => sym::null.into(),
                    };
                    PropertyValue::static_default(Value::string(sub))
                })
                .collect();

            let groups = Array::from_vec(groups, scope);
            Ok(Value::object(scope.register(groups)))
        }
        Err(_) => {
            if is_global {
                last_index.set(0);
            }
            Ok(Value::null())
        }
    }
}
