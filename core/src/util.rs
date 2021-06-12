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

#[derive(Debug, Clone)]
pub enum MaybeOwned<T> {
    Owned(T),
    Borrowed(*mut T),
}

impl<T> MaybeOwned<T> {
    pub fn as_ptr(&mut self) -> *mut T {
        match self {
            Self::Owned(v) => v as *mut T,
            Self::Borrowed(v) => *v,
        }
    }

    pub fn as_borrowed(&mut self) -> Self {
        Self::Borrowed(self.as_ptr())
    }

    pub unsafe fn as_ref(&self) -> &T {
        match self {
            Self::Borrowed(b) => &**b,
            Self::Owned(b) => b,
        }
    }

    pub unsafe fn as_mut(&mut self) -> &mut T {
        match self {
            Self::Borrowed(b) => &mut **b,
            Self::Owned(b) => b,
        }
    }
}
