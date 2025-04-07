use dash_middle::interner::sym;
use dash_regex::Flags;

use crate::localscope::LocalScope;
use crate::throw;
use crate::value::array::{Array, ArrayIterator};
use crate::value::boxed::String as BoxedString;
use crate::value::function::native::CallContext;
use crate::value::object::{OrdObject, Object, PropertyValue};
use crate::value::ops::conversions::ValueConversion;
use crate::value::regex::RegExp;
use crate::value::{Value, ValueContext};
use std::cmp;
use std::fmt::Write;
use std::ops::Range;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = match cx.args.first() {
        Some(arg) => arg.to_js_string(cx.scope)?,
        None => sym::empty.into(),
    };

    if let Some(new_target) = cx.new_target {
        let boxed = BoxedString::with_obj(value, OrdObject::instance_for_new_target(new_target, cx.scope)?);
        Ok(Value::object(cx.scope.register(boxed)))
    } else {
        Ok(Value::string(value))
    }
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    if let Some(value) = cx
        .this
        .internal_slots(cx.scope)
        .and_then(|prim| prim.string_value(cx.scope))
    {
        Ok(Value::string(value))
    } else {
        throw!(
            cx.scope,
            TypeError,
            "String.prototype.toString called on non-string value"
        )
    }
}

fn create_html(
    sc: &mut LocalScope,
    string: Value,
    tag: &str,
    attribute: Option<(&str, Value)>,
) -> Result<Value, Value> {
    // 2. Let S be ? ToString(str).
    let s = string.to_js_string(sc)?;

    // 3. Let p1 be the string-concatenation of "<" and tag.
    let mut p1 = format!("<{tag}");

    // 4. If attribute is not the empty String, then
    if let Some((key, value)) = attribute {
        // Let V be ? ToString(value).
        let v = value.to_js_string(sc)?;

        // b. Let escapedV be the String value that is  ...
        let escaped_v = v.res(sc).replace('"', "&quot;");

        // c. Set p1 to the string-concatenation of: ...
        let _ = write!(p1, " {key}=\"{escaped_v}\"");
    }

    // 5. Let p2 be the string-concatenation of p1 and ">".
    // 6. Let p3 be the string-concatenation of p2 and S.
    // Let p4 be the string-concatenation of p3, "</", tag, and ">".
    // 8. Return p4.
    let _ = write!(p1, ">{}</{tag}>", s.res(sc));

    Ok(Value::string(sc.intern(p1).into()))
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
    let this = cx.this.to_js_string(cx.scope)?.res(cx.scope);
    // TODO: this isn't right, but it is what it is
    match this.as_bytes().get(index) {
        Some(&c) => Ok(Value::string(cx.scope.intern_char(c as char).into())),
        None => Ok(Value::undefined()),
    }
}

pub fn char_code_at(cx: CallContext) -> Result<Value, Value> {
    let index = cx.args.first().unwrap_or_undefined().to_number(cx.scope)? as usize;
    let this = cx.this.to_js_string(cx.scope)?.res(cx.scope);
    // TODO: this isn't right, but it is what it is
    match this.as_bytes().get(index) {
        Some(&c) => Ok(Value::number(c as f64)),
        None => Ok(Value::undefined()),
    }
}

pub fn concat(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_js_string(cx.scope)?;

    let concat = if cx.args.len() > 1 {
        // avoid interning every concatenation
        let mut concat = String::from(this.res(cx.scope));
        for value in &cx.args {
            concat += value.to_js_string(cx.scope)?.res(cx.scope);
        }
        concat
    } else {
        let other = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
        String::from(this.res(cx.scope)) + other.res(cx.scope)
    };

    Ok(Value::string(cx.scope.intern(concat.as_ref()).into()))
}

pub fn ends_with(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_js_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    Ok(Value::boolean(this.res(cx.scope).ends_with(other.res(cx.scope))))
}

pub fn starts_with(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_js_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    Ok(Value::boolean(this.res(cx.scope).starts_with(other.res(cx.scope))))
}

pub fn includes(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_js_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    Ok(Value::boolean(this.res(cx.scope).contains(other.res(cx.scope))))
}

pub fn index_of(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_js_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    let start_index = match cx.args.get(1) {
        Some(n) => n.to_length_u(cx.scope)?,
        None => 0,
    };
    let this = this.res(cx.scope);
    let pos = this
        .get(start_index..)
        .unwrap_or("")
        .find(other.res(cx.scope))
        .map(|i| (start_index + i) as f64)
        .unwrap_or(-1.0);
    Ok(Value::number(pos))
}

pub fn last_index_of(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.to_js_string(cx.scope)?;
    let other = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    let end_index = match cx.args.get(1) {
        Some(n) => n.to_length_u(cx.scope)? + 1,
        None => this.res(cx.scope).len(),
    };

    let this = this.res(cx.scope);
    let pos = this
        .get(..end_index)
        .unwrap_or(this)
        .rfind(other.res(cx.scope))
        .map(|i| i as f64)
        .unwrap_or(-1.0);
    Ok(Value::number(pos))
}

enum PadPlacement {
    Start,
    End,
}

fn string_pad(cx: CallContext, placement: PadPlacement) -> Result<Value, Value> {
    // 1. Let S be ? ToString(O).
    let s = cx.this.to_js_string(cx.scope)?;

    // 2. Let intMaxLength be ℝ(? ToLength(maxLength)).
    let int_max_length = cx.args.get(1).unwrap_or_undefined().to_length_u(cx.scope)?;

    // 3. Let stringLength be the length of S.
    let string_length = s.res(cx.scope).len();

    // If intMaxLength ≤ stringLength, return S.
    if int_max_length <= string_length {
        return Ok(Value::string(s));
    }

    // 5. If fillString is undefined, let filler be the String value consisting solely of the code unit 0x0020 (SPACE).
    let filler = if let Some(filler) = cx.args.get(2) {
        // Else, let filler be ? ToString(fillString).
        let filler = filler.to_js_string(cx.scope)?.res(cx.scope);

        // 7. If filler is the empty String, return S.
        if filler.is_empty() {
            return Ok(Value::string(s));
        }

        filler
    } else {
        " "
    };

    // 8. Let fillLen be intMaxLength - stringLength.
    let fill_len = int_max_length - string_length;

    // 9. Let truncatedStringFiller be the String value consisting of repeated concatenations of filler truncated to length fillLen.
    let truncated_string_filler = filler.repeat(fill_len);

    // 10. If placement is start, return the string-concatenation of truncatedStringFiller and S.
    // Else, return the string-concatenation of S and truncatedStringFiller.
    let string = match placement {
        PadPlacement::Start => truncated_string_filler + s.res(cx.scope),
        PadPlacement::End => String::from(s.res(cx.scope)) + &truncated_string_filler,
    };
    Ok(Value::string(cx.scope.intern(string.as_ref()).into()))
}

pub fn pad_end(cx: CallContext) -> Result<Value, Value> {
    string_pad(cx, PadPlacement::End)
}

pub fn pad_start(cx: CallContext) -> Result<Value, Value> {
    string_pad(cx, PadPlacement::Start)
}

pub fn repeat(cx: CallContext) -> Result<Value, Value> {
    // 1. Let O be ? ToString(string).
    let o = cx.this.to_js_string(cx.scope)?;

    // 2. Let n be ? ToInteger(times).
    let n = cx.args.first().unwrap_or_undefined().to_integer_or_infinity(cx.scope)?;

    // 3. If n < 0, throw a RangeError exception.
    if n < 0.0 {
        throw!(cx.scope, RangeError, "Invalid count value");
    }

    // 4. Let result be the String value that is the concatenation of n copies of O.
    let result = o.res(cx.scope).repeat(n as usize);

    // 5. Return result.
    Ok(Value::string(cx.scope.intern(result).into()))
}

pub fn replace(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;

    let search_string = cx.args.first().unwrap_or_undefined();
    let replace_value = cx.args.get(1).unwrap_or_undefined().to_js_string(cx.scope)?;

    let string = if let Some(regex) = search_string.extract::<RegExp>(cx.scope) {
        let Some(inner_regex) = regex.inner() else {
            throw!(cx.scope, TypeError, "invalid regex object")
        };

        let replace_string = replace_value.res(cx.scope);

        let mut rest = string.res(cx.scope);
        let mut output = String::with_capacity(rest.len());
        while let Ok(res) = inner_regex.regex.eval(rest) {
            let Range { start, end } = res.full_match();

            output.push_str(&rest[..start as usize]);
            output.push_str(replace_string);

            rest = &rest[end as usize..];
            if !inner_regex.regex.flags().contains(Flags::GLOBAL) {
                break;
            }
        }
        output.push_str(rest);

        cx.scope.intern(output).into()
    } else {
        let search_string = search_string.to_js_string(cx.scope)?;

        let string = string
            .res(cx.scope)
            .replacen(search_string.res(cx.scope), replace_value.res(cx.scope), 1);
        cx.scope.intern(string).into()
    };

    Ok(Value::string(string))
}

pub fn replace_all(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;

    let search_string = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;

    let replace_value = cx.args.get(1).unwrap_or_undefined().to_js_string(cx.scope)?;

    let string = string
        .res(cx.scope)
        .replace(search_string.res(cx.scope), replace_value.res(cx.scope));

    Ok(Value::string(cx.scope.intern(string).into()))
}

pub fn split(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?.res(cx.scope).to_owned();
    let separator = cx
        .args
        .first()
        .unwrap_or_undefined()
        .to_js_string(cx.scope)?
        .res(cx.scope)
        .to_owned();

    let result = if separator.is_empty() {
        string
            .chars()
            .map(|c| PropertyValue::static_default(Value::string(cx.scope.intern_char(c).into())))
            .collect()
    } else {
        string
            .split(&separator)
            .map(|s| PropertyValue::static_default(Value::string(cx.scope.intern(s).into())))
            .collect()
    };

    let array = Array::from_vec(result, cx.scope);
    Ok(cx.scope.register(array).into())
}

pub fn to_uppercase(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;
    let result = string.res(cx.scope).to_uppercase();
    Ok(Value::string(cx.scope.intern(result).into()))
}

pub fn to_lowercase(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;
    let result = string.res(cx.scope).to_lowercase();
    Ok(Value::string(cx.scope.intern(result).into()))
}

pub fn trim(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;
    let result = string.res(cx.scope).trim().to_owned();
    Ok(Value::string(cx.scope.intern(result.as_ref()).into()))
}

pub fn trim_start(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;
    let result = string.res(cx.scope).trim_start().to_owned();
    Ok(Value::string(cx.scope.intern(result.as_ref()).into()))
}

pub fn trim_end(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;
    let result = string.res(cx.scope).trim_end().to_owned();
    Ok(Value::string(cx.scope.intern(result.as_ref()).into()))
}

pub fn from_char_code(cx: CallContext) -> Result<Value, Value> {
    let code = cx.args.first().unwrap_or_undefined().to_int32(cx.scope)?;
    let s = char::from_u32(code as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
    Ok(Value::string(cx.scope.intern_char(s).into()))
}

pub fn substr(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;
    let (start, end) = {
        let start = match cx.args.first() {
            Some(arg) => {
                let num = arg.to_int32(cx.scope)?;
                if num < 0 {
                    (num + string.len(cx.scope) as i32) as usize
                } else {
                    num as usize
                }
            }
            None => 0,
        };
        let end = match cx.args.get(1) {
            Some(arg) => arg.to_int32(cx.scope)? as usize,
            None => string.len(cx.scope),
        };

        (start, start + end)
    };

    let end = end.min(string.len(cx.scope));

    let bytes = string.res(cx.scope).as_bytes().get(start..end).unwrap_or(&[]);
    let result = String::from_utf8_lossy(bytes).into_owned();

    Ok(Value::string(cx.scope.intern(result.as_ref()).into()))
}

pub fn substring(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;
    let (mut start, mut end) = {
        let start = match cx.args.first() {
            Some(arg) => arg.to_int32(cx.scope)?.max(0) as usize,
            None => 0,
        };
        let end = match cx.args.get(1) {
            Some(arg) => (arg.to_int32(cx.scope)? as usize).min(string.len(cx.scope)),
            None => string.len(cx.scope),
        };

        (start, end)
    };

    if start > end {
        std::mem::swap(&mut start, &mut end);
    }

    let bytes = string.res(cx.scope).as_bytes().get(start..end).unwrap_or(&[]);
    let result = String::from_utf8_lossy(bytes).into_owned();

    Ok(Value::string(cx.scope.intern(result.as_ref()).into()))
}

pub fn slice(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?;
    let len = string.len(cx.scope);

    // 4. Let intStart be ? ToIntegerOrInfinity(start).
    // 5. If intStart = -∞, let from be 0.
    // 6. Else if intStart < 0, let from be max(len + intStart, 0).
    // 7. Else, let from be min(intStart, len).
    let int_start = cx.args.first().unwrap_or_undefined().to_integer_or_infinity(cx.scope)?;
    let from = if int_start == f64::NEG_INFINITY {
        0
    } else if int_start < 0. {
        cmp::max(len as isize + int_start as isize, 0) as usize
    } else {
        cmp::min(int_start as usize, len)
    };
    // 8. If end is undefined, let intEnd be len; else let intEnd be ? ToIntegerOrInfinity(end).
    let int_end = match cx.args.get(1) {
        Some(v) => v.to_integer_or_infinity(cx.scope)?,
        None => len as f64,
    };
    // 9. If intEnd = -∞, let to be 0.
    // 10. Else if intEnd < 0, let to be max(len + intEnd, 0).
    // 11. Else, let to be min(intEnd, len).
    let to = if int_end == f64::NEG_INFINITY {
        0
    } else if int_end < 0. {
        cmp::max(len as isize + int_end as isize, 0) as usize
    } else {
        cmp::min(int_end as usize, len)
    };

    if from >= to {
        Ok(Value::string(sym::empty.into()))
    } else {
        let bytes = string.res(cx.scope).as_bytes().get(from..to).unwrap_or(&[]);
        let result = String::from_utf8_lossy(bytes).into_owned();
        Ok(Value::string(cx.scope.intern(result).into()))
    }
}

pub fn iterator(cx: CallContext) -> Result<Value, Value> {
    let string = cx.this.to_js_string(cx.scope)?.res(cx.scope).to_owned();
    let chars = string
        .chars()
        .map(|c| cx.scope.intern_char(c).into())
        .map(Value::string)
        .map(PropertyValue::static_default)
        .collect::<Vec<_>>();
    let chars = Array::from_vec(chars, cx.scope);
    let chars = cx.scope.register(chars);
    let iter = ArrayIterator::new(cx.scope, Value::object(chars))?;
    let iter = cx.scope.register(iter);

    Ok(Value::object(iter))
}
