#[cfg(feature = "format")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub lo: u32,
    pub hi: u32,
}

impl Span {
    /// A dummy span that can be used for AST nodes that are generated by the compiler.
    /// These cannot be resolved and attempting to do so will cause a panic.
    /// So, be careful when emitting diagnostics that may contain these.
    /// Always prefer a real span from code that the user wrote.
    pub const COMPILER_GENERATED: Span = Span {
        lo: u32::MAX,
        hi: u32::MAX,
    };

    /// Used to check if this span is a "user" span, i.e. was written by the user
    /// and not generated by the compiler or other dummy spans.
    pub fn is_user_span(self) -> bool {
        self.lo != u32::MAX
    }

    pub fn res(self, src: &str) -> &str {
        debug_assert!(self.is_user_span()); // cannot resolve phantom/compiler spans

        &src[self.lo as usize..self.hi as usize]
    }
    pub fn to(self, other: Span) -> Span {
        debug_assert!(other.hi >= self.lo && self.is_user_span() && other.is_user_span());
        Span {
            lo: self.lo,
            hi: other.hi,
        }
    }
}

pub struct SourceMap<'buf>(&'buf str);

impl<'buf> SourceMap<'buf> {
    pub fn new(buf: &'buf str) -> Self {
        Self(buf)
    }

    pub fn resolve(&self, span: Span) -> &'buf str {
        span.res(self.0)
    }
}
