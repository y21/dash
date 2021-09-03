use std::borrow::Cow;

use crate::gc::Handle;
use crate::js_std::error;
use crate::vm::abstractions;
use crate::vm::value::{function::CallContext, Value};

/// The string constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-string-constructor
pub fn string_constructor(_args: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// Implements String.prototype.charAt
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.charat
pub fn char_at(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    // 2. Let S be ? ToString(O).
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    // Let position be ? ToIntegerOrInfinity(pos).
    let position = {
        let maybe_pos = ctx.args.first().map(|x| unsafe { x.borrow_unbounded() });
        abstractions::object::to_integer_or_infinity(maybe_pos.as_ref().map(|x| &***x))?
    };

    // Let size be the length of S.
    let size = this_s.len();

    // If position < 0 or position ≥ size, return the empty String.
    if position < 0f64 || position >= size as f64 {
        return Ok(ctx.vm.create_js_value(String::new()).into_handle(ctx.vm));
    }

    // 6. Return the String value of length 1, containing one code unit from S, namely the code unit at index position.
    let bytes = this_s.as_bytes();
    // TODO: This is not correct. This only works if chars up to `position` are in the range 0-255.
    let ret = String::from(bytes[position as usize] as char);
    Ok(ctx.vm.create_js_value(ret).into_handle(ctx.vm))
}

/// Implements String.prototype.charCodeAt
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.charcodeat
pub fn char_code_at(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    // 2. Let S be ? ToString(O).
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    // Let position be ? ToIntegerOrInfinity(pos).
    let position = {
        let maybe_pos = ctx.args.first().map(|x| unsafe { x.borrow_unbounded() });
        abstractions::object::to_integer_or_infinity(maybe_pos.as_ref().map(|x| &***x))?
    };

    // Let size be the length of S.
    let size = this_s.len();

    // If position < 0 or position ≥ size, return the empty String.
    if position < 0f64 || position >= size as f64 {
        return Ok(ctx.vm.create_js_value(String::new()).into_handle(ctx.vm));
    }

    // 6. Return the Number value for the numeric value of the code unit at index position within the String S.
    let bytes = this_s.as_bytes();
    let ret = bytes[position as usize] as f64;
    Ok(ctx.vm.create_js_value(ret).into_handle(ctx.vm))
}

/// Implements String.prototype.endsWith
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.endswith
pub fn ends_with(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    let (search_str_cell, _end_position) = {
        let mut arguments = ctx.arguments();

        let search_str = arguments.next();
        let end_position_ref = arguments.next().map(|x| unsafe { x.borrow_unbounded() });
        let end_position =
            abstractions::object::to_integer_or_infinity(end_position_ref.as_ref().map(|x| &***x))?;

        (search_str.cloned(), end_position)
    };

    let search_str_cell = abstractions::conversions::to_string(ctx.vm, search_str_cell.as_ref())?;
    let search_str_ref = unsafe { search_str_cell.borrow_unbounded() };
    let search_str = search_str_ref.as_string().unwrap();

    let ret = this_s.ends_with(search_str);
    Ok(ctx.vm.create_js_value(ret).into_handle(ctx.vm))
}

/// Implements the abstract operation CreateHTML
///
/// https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-createhtml
fn create_html(
    ctx: CallContext,
    tag: &str,
    attribute: Option<&str>,
) -> Result<Handle<Value>, Handle<Value>> {
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_str = this_ref.as_string().unwrap();

    let mut p1 = format!("<{}", tag);

    if let Some(attribute) = attribute {
        let name_cell = abstractions::conversions::to_string(ctx.vm, ctx.args.first())?;
        let name_ref = unsafe { name_cell.borrow_unbounded() };
        let name_str = name_ref.as_string().unwrap().replace("\"", "&quot;");
        p1.push(' ');
        p1.push_str(attribute);
        p1.push_str("=\"");
        p1.push_str(&name_str);
        p1.push('"');
    }

    p1.push('>');
    p1.push_str(this_str);
    p1.push_str("</");
    p1.push_str(tag);
    p1.push('>');

    Ok(ctx.vm.create_js_value(p1).into_handle(ctx.vm))
}

/// Implements String.prototype.anchor
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.anchor
pub fn anchor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "a", Some("name"))
}

/// Implements String.prototype.big
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.big
pub fn big(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "big", None)
}

/// Implements String.prototype.blink
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.blink
pub fn blink(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "blink", None)
}

/// Implements String.prototype.bold
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.bold
pub fn bold(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "b", None)
}

/// Implements String.prototype.fixed
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.fixed
pub fn fixed(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "tt", None)
}

/// Implements String.prototype.fontcolor
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.fontcolor
pub fn fontcolor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "font", Some("color"))
}

/// Implements String.prototype.fontsize
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.fontsize
pub fn fontsize(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "font", Some("size"))
}

/// Implements String.prototype.italics
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.italics
pub fn italics(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "i", None)
}

/// Implements String.prototype.link
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.link
pub fn link(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "a", Some("href"))
}

/// Implements String.prototype.small
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.small
pub fn small(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "small", None)
}

/// Implements String.prototype.strike
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.strike
pub fn strike(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "strike", None)
}

/// Implements String.prototype.sub
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.sub
pub fn sub(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "sub", None)
}

/// Implements String.prototype.sup
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.sup
pub fn sup(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    create_html(ctx, "sup", None)
}

/// Implements String.prototype.indexOf
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.indexof
pub fn index_of(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let hay = ctx.receiver.as_ref();
    let (needle, pos) = {
        let mut iter = ctx.arguments();

        (iter.next().cloned(), iter.next().cloned())
    };

    let idx = abstractions::object::index_of(ctx.vm, hay, needle.as_ref(), pos.as_ref())?;

    Ok(ctx.vm.create_js_value(idx).into_handle(ctx.vm))
}

/// Implements String.prototype.includes
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.includes
pub fn includes(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let hay = ctx.receiver.as_ref();
    let (needle, pos) = {
        let mut iter = ctx.arguments();

        (iter.next().cloned(), iter.next().cloned())
    };

    let idx = abstractions::object::index_of(ctx.vm, hay, needle.as_ref(), pos.as_ref())?;

    Ok(ctx.vm.create_js_value(idx != -1f64).into_handle(ctx.vm))
}

enum StringPadKind {
    Start, // String.prototype.padStart
    End,   // String.prototype.padEnd
}

fn string_pad(ctx: CallContext, pad_kind: StringPadKind) -> Result<Handle<Value>, Handle<Value>> {
    // 1. Let S be ? ToString(O).
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    let (max_length, fill_string) = {
        let mut iter = ctx.arguments();

        (iter.next().cloned(), iter.next().cloned())
    };

    // 2. Let intMaxLength be ℝ(? ToLength(maxLength)).
    let int_max_length = max_length.as_ref().map(|x| unsafe { x.borrow_unbounded() });
    let int_max_length =
        abstractions::object::to_length(int_max_length.as_ref().map(|x| &***x))? as usize;

    // 3. Let stringLength be the length of S.
    let string_length = this_s.len();

    // 4. If intMaxLength ≤ stringLength, return S.
    if int_max_length <= string_length {
        return Ok(Handle::clone(&this));
    }

    // 5. If fillString is undefined, let filler be the String value consisting solely of the code unit 0x0020 (SPACE).
    // 6. Else, let filler be ? ToString(fillString).
    let filler = if let Some(filler) = fill_string {
        let handle = abstractions::conversions::to_string(ctx.vm, Some(&filler))?;
        let value = unsafe { handle.borrow_unbounded() };
        Cow::Owned(unsafe { value.as_string().unwrap().to_owned() })
    } else {
        Cow::Borrowed(" ")
    };

    // 7. If filler is the empty String, return S.
    if filler.is_empty() {
        return Ok(Handle::clone(&this));
    }

    // 8. Let fillLen be intMaxLength - stringLength.
    let fill_len = int_max_length - string_length;

    // 9. Let truncatedStringFiller be the String value consisting of repeated concatenations of filler truncated to length fillLen.
    let mut truncated_string_filler = String::new();

    let filler_raw = filler.as_bytes();

    loop {
        let len = fill_len - truncated_string_filler.len();

        let filler_len = filler_raw.len();

        let truncated_filler = &filler_raw[0..len.min(filler_len)];

        let truncated_filler = String::from_utf8_lossy(truncated_filler);

        truncated_string_filler += &truncated_filler;

        if len < filler_len {
            break;
        }
    }

    // 10. If placement is start, return the string-concatenation of truncatedStringFiller and S.
    // 11. Else, return the string-concatenation of S and truncatedStringFiller.
    let result = match pad_kind {
        StringPadKind::Start => truncated_string_filler + this_s,
        StringPadKind::End => this_s.to_owned() + &truncated_string_filler,
    };

    Ok(ctx.vm.create_js_value(result).into_handle(ctx.vm))
}

/// Implements String.prototype.padStart
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.padstart
pub fn pad_start(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    string_pad(ctx, StringPadKind::Start)
}

/// Implements String.prototype.padEnd
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.padend
pub fn pad_end(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    string_pad(ctx, StringPadKind::End)
}

/// Implements String.prototype.repeat
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.repeat
pub fn repeat(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    // 2. Let S be ? ToString(O).
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    // 3. Let n be ? ToIntegerOrInfinity(count).
    let n = {
        let count = ctx.args.first().map(|x| unsafe { x.borrow_unbounded() });
        abstractions::object::to_integer_or_infinity(count.as_ref().map(|x| &***x))?
    };

    // 4. If n < 0 or n is +∞, throw a RangeError exception.
    if n < 0f64 || n.is_infinite() {
        return Err(error::create_error("Invalid count value".into(), ctx.vm));
    }

    // 5. If n is 0, return the empty String.
    if n == 0f64 {
        return Ok(ctx.vm.create_js_value(String::new()).into_handle(ctx.vm));
    }

    // 6. Return the String value that is made from n copies of S appended together.
    let result = this_s.repeat(n as usize);

    Ok(ctx.vm.create_js_value(result).into_handle(ctx.vm))
}

/// Implements String.prototype.toLowerCase
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.tolowercase
pub fn to_lowercase(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    // 2. Let S be ? ToString(O).
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    let result = this_s.to_lowercase();

    Ok(ctx.vm.create_js_value(result).into_handle(ctx.vm))
}

/// Implements String.prototype.toUpperCase
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.touppercase
pub fn to_uppercase(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    // 2. Let S be ? ToString(O).
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    let result = this_s.to_uppercase();

    Ok(ctx.vm.create_js_value(result).into_handle(ctx.vm))
}

/// Implements String.prototype.replace
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.replace
pub fn replace(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this = abstractions::conversions::to_string(ctx.vm, ctx.receiver.as_ref())?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    let (search_value, replace_value) = {
        let mut iter = ctx.arguments();

        (iter.next().cloned(), iter.next().cloned())
    };

    let search_value = abstractions::conversions::to_string(ctx.vm, search_value.as_ref())?;
    let search_value_ref = unsafe { search_value.borrow_unbounded() };
    let search_value_s = search_value_ref.as_string().unwrap();
    let replace_value_ref = replace_value
        .as_ref()
        .map(|x| unsafe { x.borrow_unbounded() });

    let functional_replace = replace_value_ref
        .as_ref()
        .map(|x| x.is_callable())
        .unwrap_or_default();

    let replacer = if functional_replace {
        let this = replace_value.as_ref().unwrap();
        Value::call(this, Vec::new(), ctx.vm)?
    } else {
        abstractions::conversions::to_string(ctx.vm, replace_value.as_ref())?
    };

    let replacer_ref = unsafe { replacer.borrow_unbounded() };
    let replacer_s = replacer_ref.as_string().unwrap();

    let result = this_s.replacen(search_value_s, replacer_s, 1);
    Ok(ctx.vm.create_js_value(result).into_handle(ctx.vm))
}
