pub fn is_digit(c: u8) -> bool {
    (b'0'..=b'9').contains(&c)
}

pub fn is_alpha(c: u8) -> bool {
    (b'a'..=b'z').contains(&c) || (b'A'..=b'Z').contains(&c) || c == b'_'
}

/// Checks if `c` is a valid character for the start of an identifier
pub fn is_identifier_start(c: u8) -> bool {
    is_alpha(c)
}

pub fn is_numeric(c: impl AsRef<str>) -> bool {
    c.as_ref().chars().all(|c| c.is_numeric())
}
