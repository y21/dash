use crate::gc::interner::sym;
use crate::throw;
use crate::value::array::Array;
use crate::value::function::native::CallContext;
use crate::value::object::PropertyValue;
use crate::value::ops::conversions::ValueConversion;
use crate::value::regex::{RegExp, RegExpInner};
use crate::value::{Value, ValueContext};
use dash_regex::matcher::Matcher as RegexMatcher;
use dash_regex::parser::Parser as RegexParser;
use dash_regex::Flags;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let pattern = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    let flags = match cx
        .args
        .get(1)
        .map(|v| v.to_js_string(cx.scope))
        .transpose()?
        .map(|s| s.res(cx.scope).parse::<Flags>())
    {
        Some(Ok(flags)) => flags,
        Some(Err(err)) => throw!(cx.scope, SyntaxError, "Invalid RegExp flags: {:?}", err),
        None => Flags::empty(),
    };

    let nodes = match RegexParser::new(pattern.res(cx.scope).as_bytes()).parse_all() {
        Ok(nodes) => nodes,
        Err(err) => throw!(cx.scope, SyntaxError, "Regex parser error: {}", err),
    };

    let regex = RegExp::new(nodes, flags, pattern, cx.scope);

    Ok(Value::Object(cx.scope.register(regex)))
}

pub fn test(cx: CallContext) -> Result<Value, Value> {
    let text = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;

    let regex = match cx.this.downcast_ref::<RegExp>() {
        Some(regex) => regex,
        None => throw!(cx.scope, TypeError, "Receiver must be a RegExp"),
    };

    let RegExpInner {
        regex,
        last_index,
        flags,
        ..
    } = match regex.inner() {
        Some(nodes) => nodes,
        None => throw!(cx.scope, TypeError, "Receiver must be an initialized RegExp object"),
    };

    let text = text.res(cx.scope);
    let is_global = flags.contains(Flags::GLOBAL);

    if is_global && last_index.get() >= text.len() {
        last_index.set(0);
        return Ok(Value::Boolean(false));
    }

    let mut matcher = RegexMatcher::new(regex, text[last_index.get()..].as_bytes());
    if matcher.matches() {
        if is_global {
            last_index.set(last_index.get() + matcher.groups.get(0).unwrap().end);
        }
        Ok(Value::Boolean(true))
    } else {
        if is_global {
            last_index.set(0);
        }
        Ok(Value::Boolean(false))
    }
}

pub fn exec(cx: CallContext<'_, '_>) -> Result<Value, Value> {
    let text = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;

    let regex = match cx.this.downcast_ref::<RegExp>() {
        Some(regex) => regex,
        None => throw!(cx.scope, TypeError, "Receiver must be a RegExp"),
    };

    let RegExpInner {
        regex,
        last_index,
        flags,
        ..
    } = match regex.inner() {
        Some(nodes) => nodes,
        None => throw!(cx.scope, TypeError, "Receiver must be an initialized RegExp object"),
    };

    let text = text.res(cx.scope).to_owned();
    let is_global = flags.contains(Flags::GLOBAL);

    if is_global && last_index.get() >= text.len() {
        last_index.set(0);
        return Ok(Value::null());
    }

    let mut matcher = RegexMatcher::new(regex, text[last_index.get()..].as_bytes());
    if matcher.matches() {
        if is_global {
            last_index.set(last_index.get() + matcher.groups.get(0).unwrap().end);
        }

        let groups = matcher
            .groups
            .iter()
            .map(|g| {
                let sub = match g {
                    Some(r) => cx.scope.intern(&text[r]).into(),
                    None => sym::NULL.into(),
                };
                PropertyValue::static_default(Value::String(sub))
            })
            .collect();

        let groups = Array::from_vec(cx.scope, groups);
        Ok(Value::Object(cx.scope.register(groups)))
    } else {
        if is_global {
            last_index.set(0);
        }

        Ok(Value::null())
    }
}
