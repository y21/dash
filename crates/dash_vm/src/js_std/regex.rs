use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::regex::RegExp;
use crate::value::{Value, ValueContext};
use dash_regex::matcher::Matcher as RegexMatcher;
use dash_regex::parser::Parser as RegexParser;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let pattern = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;

    let nodes = match RegexParser::new(pattern.as_bytes()).parse_all() {
        Ok(nodes) => nodes,
        Err(err) => throw!(cx.scope, SyntaxError, "Regex parser error: {}", err),
    };

    let regex = RegExp::new(nodes, pattern, cx.scope);

    Ok(Value::Object(cx.scope.register(regex)))
}

pub fn test(cx: CallContext) -> Result<Value, Value> {
    let text = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;

    let regex = match cx.this.downcast_ref::<RegExp>() {
        Some(regex) => regex,
        None => throw!(cx.scope, TypeError, "Receiver must be a RegExp"),
    };

    let (nodes, _) = match regex.inner() {
        Some(nodes) => nodes,
        None => throw!(cx.scope, TypeError, "Receiver must be an initialized RegExp object"),
    };

    let mut matcher = RegexMatcher::new(nodes, text.as_bytes());
    Ok(Value::Boolean(matcher.matches()))
}
