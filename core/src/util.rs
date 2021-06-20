pub fn is_digit(c: u8) -> bool {
    (b'0'..=b'9').contains(&c)
}

pub fn is_alpha(c: u8) -> bool {
    is_identifier_start(c) || is_digit(c)
}

/// Checks if `c` is a valid character for the start of an identifier
pub fn is_identifier_start(c: u8) -> bool {
    (b'a'..=b'z').contains(&c) || (b'A'..=b'Z').contains(&c) || c == b'_' || c == b'$'
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

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn as_left(&self) -> Option<&L> {
        match self {
            Self::Left(l) => Some(l),
            Self::Right(_) => None,
        }
    }

    pub fn as_left_or_else<F>(&self, f: F) -> Option<&L>
    where
        F: FnOnce(&R) -> Option<&L>,
    {
        match self {
            Self::Left(l) => Some(l),
            Self::Right(r) => f(r),
        }
    }

    pub fn as_right(&self) -> Option<&R> {
        match self {
            Self::Left(_) => None,
            Self::Right(r) => Some(r),
        }
    }

    pub fn as_right_or_else<F>(&self, f: F) -> Option<&R>
    where
        F: FnOnce(&L) -> Option<&R>,
    {
        match self {
            Self::Left(l) => f(l),
            Self::Right(r) => Some(r),
        }
    }

    pub fn into_left(self) -> Option<L> {
        match self {
            Self::Left(l) => Some(l),
            Self::Right(_) => None,
        }
    }

    pub fn into_right(self) -> Option<R> {
        match self {
            Self::Left(_) => None,
            Self::Right(r) => Some(r),
        }
    }
}
