use std::borrow::Cow;
use std::collections::HashMap;
use std::num::ParseFloatError;
use std::str::Utf8Error;

use crate::util;
use crate::vm::value::array::Array;
use crate::vm::value::Value as JsValue;
use crate::vm::value::ValueKind;
use crate::vm::VM;

/// A JSON value
#[derive(Debug)]
pub enum Value<'a> {
    /// A JSON string
    String(&'a [u8]),
    /// A JSON number
    Number(f64),
    /// A JSON bool
    Bool(bool),
    /// A JSON array (collection of JS values)
    Array(Vec<Value<'a>>),
    /// A JSON object/dictionary/map
    Object(HashMap<&'a [u8], Value<'a>>),
    /// A JSON null value
    Null,
}

/// An error that occurred during parsing JSON
///
/// If possible, variants carry an additional `usize` with them which is the offset
/// of where the error occurred
#[derive(Debug)]
pub enum JsonParseError {
    /// Unexpected end of file
    UnexpectedEof,
    /// Unexpected token
    UnexpectedToken(u8, usize),
    /// UTF8 error
    Utf8Error(Utf8Error, usize),
    /// Failed to parse a number
    ParseFloatError(ParseFloatError, usize),
}

impl JsonParseError {
    /// Tries to format a JSON error
    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::UnexpectedEof => Cow::Borrowed("Unexpected end of JSON input"),
            Self::UnexpectedToken(token, position) => Cow::Owned(format!(
                "Unexpected token {} at position {}",
                *token as char, *position
            )),
            Self::Utf8Error(_, pos) => Cow::Owned(format!("Utf8 Error at position {}", pos)),
            Self::ParseFloatError(_, pos) => {
                Cow::Owned(format!("Failed to parse number at position {}", pos))
            }
        }
    }
}

/// An error that may occur during converting
#[derive(Debug)]
pub enum ConversionError {
    /// UTF8 encoding error
    Utf8Error(Utf8Error),
}

impl ConversionError {
    /// Formats this error by calling to_string on the underlying error
    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::Utf8Error(u) => Cow::Owned(u.to_string()),
        }
    }
}

impl From<Utf8Error> for ConversionError {
    fn from(u: Utf8Error) -> Self {
        Self::Utf8Error(u)
    }
}

impl<'a> Value<'a> {
    /// Attempts to convert a JSON value to a JavaScript value
    pub(crate) fn into_js_value(self, vm: &VM) -> Result<JsValue, ConversionError> {
        match self {
            Self::String(s) => std::str::from_utf8(s)
                .map(String::from)
                .map(|s| vm.create_js_value(s))
                .map_err(Into::into),
            Self::Number(n) => Ok(vm.create_js_value(n)),
            Self::Bool(b) => Ok(vm.create_js_value(b)),
            Self::Array(arr) => Ok(vm.create_js_value(Array::new({
                let mut js_arr = Vec::with_capacity(arr.len());

                for value in arr {
                    js_arr.push(value.into_js_value(vm).map(|v| v.into_handle(vm))?);
                }

                js_arr
            }))),
            Self::Object(obj) => {
                let mut js_obj = vm.create_object();

                for (key, value) in obj {
                    let key = std::str::from_utf8(key)?;
                    js_obj.set_property(
                        String::from(key).into_boxed_str(),
                        value.into_js_value(vm)?.into_handle(vm),
                    );
                }

                Ok(js_obj)
            }
            Self::Null => Ok(JsValue::new(ValueKind::Null)),
        }
    }
}

/// A tiny, zero-copy JSON parser that borrows from the input string
pub struct Parser<'a> {
    source: &'a [u8],
    idx: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new JSON parser
    pub fn new(source: &'a [u8]) -> Self {
        Self { source, idx: 0 }
    }

    /// Returns the current byte, if present
    fn current(&self) -> Option<u8> {
        self.source.get(self.idx).copied()
    }

    /// Skips any unnecessary tokens, such as whitespaces and returns the next relevant byte
    fn skip_to_relevant_token(&mut self) -> Option<u8> {
        self.skip_whitespaces();
        self.current()
    }

    /// Returns the next byte, if present
    fn next(&mut self) -> Option<u8> {
        self.idx += 1;
        self.current()
    }

    /// Reads a string literal starting at the current position
    fn read_string_literal(&mut self) -> Result<&'a [u8], JsonParseError> {
        let start = self.idx + 1;
        while let Some(cur) = self.next() {
            if cur == b'"' {
                self.idx += 1;
                return Ok(&self.source[start..self.idx - 1]);
            }
        }
        Err(JsonParseError::UnexpectedEof)
    }

    /// Reads a number literal at the current position
    fn read_number_literal(&mut self) -> Result<&'a [u8], JsonParseError> {
        let start = self.idx;
        let mut has_point = false;
        let mut has_expo = false;

        while let Some(cur) = self.next() {
            match cur {
                b'.' => {
                    if has_point {
                        return Err(JsonParseError::UnexpectedToken(b'.', self.idx));
                    } else {
                        has_point = true
                    }
                }
                b'e' => {
                    if has_expo {
                        return Err(JsonParseError::UnexpectedToken(b'e', self.idx));
                    } else {
                        has_expo = true
                    }
                }
                _ => {}
            }

            if !util::is_digit(cur) {
                break;
            }
        }
        Ok(&self.source[start..self.idx])
    }

    /// Skips any whitespace token
    pub fn skip_whitespaces(&mut self) {
        while let Some(cur) = self.current() {
            if ![b' ', b'\n'].contains(&cur) {
                return;
            }
            self.idx += 1;
        }
    }

    /// Parses the input string that belongs to this parser
    ///
    /// Any more calls will fail because the index will have reached the end of the string
    pub fn parse(&mut self) -> Result<Value<'a>, JsonParseError> {
        self.skip_whitespaces();
        let cur = self.current().ok_or(JsonParseError::UnexpectedEof)?;

        match cur {
            b'[' => {
                let mut arr = Vec::new();

                self.idx += 1;

                while let Some(cur) = self.skip_to_relevant_token() {
                    match cur {
                        b',' => self.idx += 1,
                        b']' => {
                            self.idx += 1;
                            break;
                        }
                        _ => {}
                    };

                    arr.push(self.parse()?);
                }

                Ok(Value::Array(arr))
            }
            b'{' => {
                let mut obj = HashMap::new();

                self.idx += 1;

                while let Some(cur) = self.skip_to_relevant_token() {
                    match cur {
                        b'}' => break,
                        b',' => self.idx += 1,
                        _ => {}
                    };

                    self.skip_whitespaces();

                    let key = self.read_string_literal()?; // "key"
                    self.skip_whitespaces(); // spaces
                    self.idx += 1; // :
                    let value = self.parse()?;
                    obj.insert(key, value);
                }

                Ok(Value::Object(obj))
            }
            b'"' => self.read_string_literal().map(Value::String),
            _ if util::is_digit(cur) => {
                let num = std::str::from_utf8(self.read_number_literal()?)
                    .map_err(|e| JsonParseError::Utf8Error(e, self.idx))?
                    .parse::<f64>()
                    .map_err(|e| JsonParseError::ParseFloatError(e, self.idx))?;
                Ok(Value::Number(num))
            }
            other => Err(JsonParseError::UnexpectedToken(other, self.idx)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn json_string() {
        let result = Parser::new(br#""hi""#).parse();

        assert!(matches!(result, Ok(Value::String(b"hi"))));
    }
}
