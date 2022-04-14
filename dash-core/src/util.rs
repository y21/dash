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

/// A T that may be either owned or borrowed
#[derive(Debug, Clone)]
pub enum MaybeOwned<T> {
    /// Owned T
    Owned(T),
    /// Borrowed T
    Borrowed(*mut T),
}

impl<T> MaybeOwned<T> {
    /// Returns a mutable pointer to T
    pub fn as_ptr(&mut self) -> *mut T {
        match self {
            Self::Owned(v) => v as *mut T,
            Self::Borrowed(v) => *v,
        }
    }

    /// Returns self as a pointer
    pub fn as_borrowed(&mut self) -> Self {
        Self::Borrowed(self.as_ptr())
    }

    /// Attempts to return self as an owned T
    pub fn into_owned(self) -> Option<T> {
        match self {
            Self::Owned(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to the T
    ///
    /// This operation is unsafe because the pointer may be invalid
    pub unsafe fn as_ref(&self) -> &T {
        match self {
            Self::Borrowed(b) => &**b,
            Self::Owned(b) => b,
        }
    }

    /// Returns a mutable reference to the T
    ///
    /// This operation is unsafe because the pointer may be invalid
    pub unsafe fn as_mut(&mut self) -> &mut T {
        match self {
            Self::Borrowed(b) => &mut **b,
            Self::Owned(b) => b,
        }
    }
}

/// An enum that can be either L or R
pub enum Either<L, R> {
    /// Left variant
    Left(L),
    /// Right variant
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Returns a reference to the L
    pub fn as_left(&self) -> Option<&L> {
        match self {
            Self::Left(l) => Some(l),
            Self::Right(_) => None,
        }
    }

    /// Returns a reference to L, or applies a predicate that
    /// must return a reference to an L
    pub fn as_left_or_else<F>(&self, f: F) -> Option<&L>
    where
        F: FnOnce(&R) -> Option<&L>,
    {
        match self {
            Self::Left(l) => Some(l),
            Self::Right(r) => f(r),
        }
    }

    /// Returns a reference to the R
    pub fn as_right(&self) -> Option<&R> {
        match self {
            Self::Left(_) => None,
            Self::Right(r) => Some(r),
        }
    }

    /// Returns a reference to R, or applies a predicate that
    /// must return a reference to an R
    pub fn as_right_or_else<F>(&self, f: F) -> Option<&R>
    where
        F: FnOnce(&L) -> Option<&R>,
    {
        match self {
            Self::Left(l) => f(l),
            Self::Right(r) => Some(r),
        }
    }

    /// Returns an owned L
    pub fn into_left(self) -> Option<L> {
        match self {
            Self::Left(l) => Some(l),
            Self::Right(_) => None,
        }
    }

    /// Returns an owned R
    pub fn into_right(self) -> Option<R> {
        match self {
            Self::Left(_) => None,
            Self::Right(r) => Some(r),
        }
    }
}

#[cold]
fn unlikely_inner() {}

pub(crate) fn unlikely(b: bool) -> bool {
    if b {
        unlikely_inner();
    }
    b
}
