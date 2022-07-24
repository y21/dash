use crate::local::LocalScope;
use crate::throw;
use crate::value::array::Array;
use crate::value::function::native::CallContext;
use crate::value::object::PropertyValue;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::Value;
use crate::value::ValueContext;
use std::borrow::Cow;
use std::fmt::Write;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.get(0).unwrap_or_undefined().to_string(cx.scope)?;
    Ok(Value::String(value))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    Ok(cx.this)
}

fn create_html(
    sc: &mut LocalScope,
    string: Value,
    tag: &str,
    attribute: Option<(&str, Value)>,
) -> Result<Value, Value> {
    // 2. Let S be ? ToString(str).
    let s = string.to_string(sc)?;

    // 3. Let p1 be the string-concatenation of "<" and tag.
    let mut p1 = format!("<{tag}");

    // 4. If attribute is not the empty String, then
    if let Some((key, value)) = attribute {
        // Let V be ? ToString(value).
        let v = value.to_string(sc)?;

        // b. Let escapedV be the String value that is  ...
        let escaped_v = v.replace('"', "&quot;");

        // c. Set p1 to the string-concatenation of: ...
        let _ = write!(p1, " {key}=\"{escaped_v}\"");
    }

    // 5. Let p2 be the string-concatenation of p1 and ">".
    // 6. Let p3 be the string-concatenation of p2 and S.
    // Let p4 be the string-concatenation of p3, "</", tag, and ">".
    // 8. Return p4.
    let _ = write!(p1, ">{s}</{tag}>");

    Ok(Value::String(p1.into()))
}

macro_rules! define_html_methods_no_attribute {
    ($($function:ident: $name:ident),*) => {
        $(
            pub fn $function(cx: CallContext) -> Result<Value, Value> {
                create_html(cx.scope, cx.this, stringify!($name), None)
            }
        )*
    };
}

macro_rules! define_html_methods_with_attribute {
    ($($function:ident: $name:ident, $attribute:ident),*) => {
        $(
            pub fn $function(cx: CallContext) -> Result<Value, Value> {
                let attribute = cx.args.first().unwrap_or_undefined();
                create_html(cx.scope, cx.this, stringify!($name), Some((stringify!($attribute), attribute)))
            }
        )*
    };
}

define_html_methods_no_attribute! {
    big: big,
    blink: blink,
    bold: b,
    fixed: tt,
    italics: i,
    strike: strike,
    sub: sub,
    sup: sup
}

define_html_methods_with_attribute! {
    fontcolor: font, color,
    fontsize: font, size,
    link: a, href
}

pub fn char_at(cx: CallContext) -> Result<Value, Value> {
    let index = cx.args.first().unwrap_or_undefined().to_number(cx.scope)? as usize;
    let this = cx.this.to_string(cx.scope)?;
    // TODO: this isn't right, but it is what it is
    match this.as_bytes().get(index) {
        Some(&c) => Ok(Value::String((c as char).to_string().into())),
        None => Ok(Value::undefined()),
    }
}

pub fn char_code_at(cx: CallContext) -> Result<Value, Value> {
    let index = cx.args.first().unwrap_or_undefined().to_number(cx.scope)? as usize;
    let this = cx.this.to_string(cx.scope)?;
    // TODO: this isn't right, but it is what it is
    match this.as_bytes().get(index) {
        Some(&c) => Ok(Value::Number(c as f64)),
        None => Ok(Value::undefined()),
    }
}

pub fn concat(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    let concat = String::from(this.as_ref()) + other.as_ref();
    Ok(Value::String(concat.into()))
}

pub fn ends_with(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    Ok(Value::Boolean(this.ends_with(other.as_ref())))
}

pub fn starts_with(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    Ok(Value::Boolean(this.starts_with(other.as_ref())))
}

pub fn includes(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    Ok(Value::Boolean(this.contains(other.as_ref())))
}

pub fn index_of(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    let pos = this.find(other.as_ref()).map(|i| i as f64).unwrap_or(-1.0);
    Ok(Value::Number(pos))
}

pub fn last_index_of(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    let pos = this.rfind(other.as_ref()).map(|i| i as f64).unwrap_or(-1.0);
    Ok(Value::Number(pos))
}

enum PadPlacement {
    Start,
    End,
}

fn string_pad(cx: CallContext, placement: PadPlacement) -> Result<Value, Value> {
    // 1. Let S be ? ToString(O).
    let s = cx.this.to_string(cx.scope)?;

    // 2. Let intMaxLength be ℝ(? ToLength(maxLength)).
    let int_max_length = cx.args.get(1).unwrap_or_undefined().to_length_u(cx.scope)?;

    // 3. Let stringLength be the length of S.
    let string_length = s.len();

    // If intMaxLength ≤ stringLength, return S.
    if int_max_length <= string_length {
        return Ok(Value::String(s));
    }

    // 5. If fillString is undefined, let filler be the String value consisting solely of the code unit 0x0020 (SPACE).
    let filler = if let Some(filler) = cx.args.get(2) {
        // Else, let filler be ? ToString(fillString).
        let filler = filler.to_string(cx.scope)?;

        // 7. If filler is the empty String, return S.
        if filler.is_empty() {
            return Ok(Value::String(s));
        }

        Cow::Owned(String::from(filler.as_ref()))
    } else {
        Cow::Borrowed(" ")
    };

    // 8. Let fillLen be intMaxLength - stringLength.
    let fill_len = int_max_length - string_length;

    // 9. Let truncatedStringFiller be the String value consisting of repeated concatenations of filler truncated to length fillLen.
    let truncated_string_filler = filler.repeat(fill_len);

    // 10. If placement is start, return the string-concatenation of truncatedStringFiller and S.
    // Else, return the string-concatenation of S and truncatedStringFiller.
    match placement {
        PadPlacement::Start => Ok(Value::String((truncated_string_filler + s.as_ref()).into())),
        PadPlacement::End => Ok(Value::String(
            (String::from(s.as_ref()) + &truncated_string_filler).into(),
        )),
    }
}

pub fn pad_end(cx: CallContext) -> Result<Value, Value> {
    string_pad(cx, PadPlacement::End)
}

pub fn pad_start(cx: CallContext) -> Result<Value, Value> {
    string_pad(cx, PadPlacement::Start)
}

pub fn repeat(cx: CallContext) -> Result<Value, Value> {
    // 1. Let O be ? ToString(string).
    let o = cx.this.to_string(cx.scope)?;

    // 2. Let n be ? ToInteger(times).
    let n = cx.args.first().unwrap_or_undefined().to_integer_or_infinity(cx.scope)?;

    // 3. If n < 0, throw a RangeError exception.
    if n < 0.0 {
        throw!(cx.scope, "Invalid count value");
    }

    // 4. Let result be the String value that is the concatenation of n copies of O.
    let result = o.repeat(n as usize);

    // 5. Return result.
    Ok(Value::String(result.into()))
}

pub fn replace(cx: CallContext) -> Result<Value, Value> {
    // TODO: once we have regexp, we can properly implement this
    let string = cx.this.to_string(cx.scope)?;

    let search_string = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;

    let replace_value = cx.args.get(1).unwrap_or_undefined().to_string(cx.scope)?;

    let string = string.replacen(search_string.as_ref(), replace_value.as_ref(), 1);

    Ok(Value::String(string.into()))
}

pub fn replace_all(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_string(cx.scope)?;

    let search_string = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;

    let replace_value = cx.args.get(1).unwrap_or_undefined().to_string(cx.scope)?;

    let string = string.replace(search_string.as_ref(), replace_value.as_ref());

    Ok(Value::String(string.into()))
}

pub fn split(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_string(cx.scope)?;
    let separator = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;

    let result = string
        .split(separator.as_ref())
        .map(|s| PropertyValue::Static(Value::String(s.into())))
        .collect();

    let array = Array::from_vec(cx.scope, result);
    Ok(cx.scope.gc_mut().register(array).into())
}

pub fn to_uppercase(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_string(cx.scope)?;
    let result = string.to_uppercase();
    Ok(Value::String(result.into()))
}

pub fn to_lowercase(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_string(cx.scope)?;
    let result = string.to_lowercase();
    Ok(Value::String(result.into()))
}
