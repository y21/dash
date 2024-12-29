use std::borrow::Cow;
use std::num::ParseFloatError;
use std::str::Utf8Error;

use dash_middle::util;

use crate::localscope::LocalScope;
use crate::value::Value;
use crate::value::array::Array;
use crate::value::object::{NamedObject, ObjectMap, PropertyValue};
use crate::value::propertykey::PropertyKey;

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
            Self::UnexpectedToken(token, position) => {
                Cow::Owned(format!("Unexpected token {} at position {}", *token as char, *position))
            }
            Self::Utf8Error(_, pos) => Cow::Owned(format!("Utf8 Error at position {}", pos)),
            Self::ParseFloatError(_, pos) => Cow::Owned(format!("Failed to parse number at position {}", pos)),
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

/// A tiny, zero-copy JSON parser that borrows from the input string
pub struct Parser<'a, 'sc, 'vm> {
    source: &'a [u8],
    idx: usize,
    sc: &'sc mut LocalScope<'vm>,
}

impl<'a, 'sc, 'vm> Parser<'a, 'sc, 'vm> {
    /// Creates a new JSON parser
    pub fn new(source: &'a [u8], sc: &'sc mut LocalScope<'vm>) -> Self {
        Self { source, idx: 0, sc }
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
    pub fn parse(&mut self) -> Result<Value, JsonParseError> {
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

                    arr.push(PropertyValue::static_default(self.parse()?));
                }

                let arr = Array::from_vec(self.sc, arr);
                Ok(Value::object(self.sc.register(arr)))
            }
            b'{' => {
                let mut obj = ObjectMap::default();

                self.idx += 1;

                while let Some(cur) = self.skip_to_relevant_token() {
                    match cur {
                        b'}' => break,
                        b',' => self.idx += 1,
                        _ => {}
                    };

                    self.skip_whitespaces();

                    let key = self.read_string_literal()?; // "key"
                    let key = self
                        .sc
                        .intern(std::str::from_utf8(key).map_err(|err| JsonParseError::Utf8Error(err, self.idx))?);

                    self.skip_whitespaces(); // spaces
                    self.idx += 1; // :
                    let value = self.parse()?;
                    obj.insert(PropertyKey::String(key.into()), PropertyValue::static_default(value));
                }

                let obj = NamedObject::with_values(self.sc, obj);

                Ok(Value::object(self.sc.register(obj)))
            }
            b'"' => {
                let string = self.read_string_literal()?;
                std::str::from_utf8(string)
                    .map_err(|err| JsonParseError::Utf8Error(err, self.idx))
                    .map(|s| Value::string(self.sc.intern(s).into()))
            }
            _ if util::is_digit(cur) => {
                let num = std::str::from_utf8(self.read_number_literal()?)
                    .map_err(|e| JsonParseError::Utf8Error(e, self.idx))?
                    .parse::<f64>()
                    .map_err(|e| JsonParseError::ParseFloatError(e, self.idx))?;
                Ok(Value::number(num))
            }
            other => Err(JsonParseError::UnexpectedToken(other, self.idx)),
        }
    }
}
