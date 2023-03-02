//! A wrapper around any logging crate
//! internal dash crates don't directly depend on logging crates such as tracing such that
//! it can very easily be changed.

#[doc(hidden)]
pub use tracing as __tracing;
#[doc(hidden)]
pub const ENABLED: bool = matches!(option_env!("DASH_LOG"), Some(..));

pub use tracing::Level;

#[macro_export]
macro_rules! debug {
    ($($tok:tt)*) => {
        if $crate::ENABLED {
            $crate::__tracing::debug!($($tok)*)
        }
    };
}

#[macro_export]
macro_rules! error {
    ($($tok:tt)*) => {
        if $crate::ENABLED {
            $crate::__tracing::error!($($tok)*)
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($tok:tt)*) => {
        if $crate::ENABLED {
            $crate::__tracing::warn!($($tok)*)
        }
    };
}

#[macro_export]
macro_rules! span {
    ($($tok:tt)*) => {
        if $crate::ENABLED {
            $crate::Span::Enabled($crate::__tracing::span!($($tok)*))
        } else {
            $crate::Span::Disabled
        }
    }
}

#[macro_export]
macro_rules! event {
    ($($tok:tt)*) => {
        if $crate::ENABLED {
            $crate::__tracing::event!($($tok)*);
        }
    };
}

pub enum Span {
    Enabled(tracing::Span),
    Disabled,
}

impl Span {
    pub fn enter(&self) -> Entered<'_> {
        match self {
            Self::Enabled(s) => Entered::Enabled(s.enter()),
            Self::Disabled => Entered::Disabled,
        }
    }

    pub fn in_scope<T, F: FnOnce() -> T>(&self, f: F) -> T {
        match self {
            Self::Enabled(s) => s.in_scope(f),
            Self::Disabled => f(),
        }
    }
}

pub enum Entered<'a> {
    Enabled(tracing::span::Entered<'a>),
    Disabled,
}
