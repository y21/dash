use std::fmt;
use std::io::Read;
use std::ops::RangeInclusive;

const DIGIT: RangeInclusive<u8> = b'0'..=b'9';
const OCTAL_DIGIT: RangeInclusive<u8> = b'0'..=b'7';
const IDENTIFIER_START_LOWERCASE: RangeInclusive<u8> = b'a'..=b'z';
const IDENTIFIER_START_UPPERCASE: RangeInclusive<u8> = b'A'..=b'Z';
const HEX_LOWERCASE: RangeInclusive<u8> = b'a'..=b'f';
const HEX_UPPERCASE: RangeInclusive<u8> = b'A'..=b'F';

/// Checks whether the `c` is a digit
pub fn is_digit(c: u8) -> bool {
    DIGIT.contains(&c)
}

/// Checks whether `c` is a valid hex digit
pub fn is_hex_digit(c: u8) -> bool {
    DIGIT.contains(&c) || HEX_LOWERCASE.contains(&c) || HEX_UPPERCASE.contains(&c)
}

/// Checks whether `c` is a valid binary digit
pub fn is_binary_digit(c: u8) -> bool {
    c == b'0' || c == b'1'
}

/// Checks whether `c` is a valid octal digit
pub fn is_octal_digit(c: u8) -> bool {
    OCTAL_DIGIT.contains(&c)
}

/// Checks whether `c` is a valid identifier
pub fn is_alpha(c: u8) -> bool {
    is_identifier_start(c) || is_digit(c)
}

/// Checks if `c` is a valid character for the start of an identifier
pub fn is_identifier_start(c: u8) -> bool {
    IDENTIFIER_START_LOWERCASE.contains(&c) || IDENTIFIER_START_UPPERCASE.contains(&c) || c == b'_' || c == b'$'
}

/// Checks if `c` is numeric
pub fn is_numeric(c: impl AsRef<str>) -> bool {
    c.as_ref().chars().all(|c| c.is_numeric())
}

#[cold]
fn unlikely_inner() {}

pub fn unlikely(b: bool) -> bool {
    if b {
        unlikely_inner();
    }
    b
}

pub fn force_utf8(s: &[u8]) -> String {
    std::str::from_utf8(s).expect("Invalid UTF8").into()
}

pub fn force_utf8_borrowed(s: &[u8]) -> &str {
    std::str::from_utf8(s).expect("Invalid UTF8")
}

pub fn fmt_group<D: fmt::Display>(formatter: &mut fmt::Formatter<'_>, items: &[D], delim: &str) -> fmt::Result {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            write!(formatter, "{}", delim)?;
        }

        write!(formatter, "{}", item)?;
    }
    Ok(())
}

pub struct Reader<R: Read>(R, usize);

impl<R: Read> Reader<R> {
    pub fn new(r: R) -> Self {
        Self(r, 0)
    }

    pub fn offset(&self) -> usize {
        self.1
    }

    pub fn read_bytes<const N: usize>(&mut self) -> Option<[u8; N]> {
        let mut buf = [0; N];
        self.0.read_exact(&mut buf).ok()?;
        self.1 += N;
        Some(buf)
    }

    pub fn read(&mut self) -> Option<u8> {
        self.read_bytes::<1>().map(|[b]| b)
    }

    pub fn read_u16_ne(&mut self) -> Option<u16> {
        self.read_bytes().map(u16::from_ne_bytes)
    }

    pub fn read_i16_ne(&mut self) -> Option<i16> {
        self.read_bytes().map(i16::from_ne_bytes)
    }
}
