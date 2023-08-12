#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub lo: u32,
    pub hi: u32,
}

impl Span {
    pub fn res(self, src: &str) -> &str {
        &src[self.lo as usize..self.hi as usize]
    }
    pub fn to(self, other: Span) -> Span {
        debug_assert!(other.hi >= self.lo);
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
        &self.0[span.lo as usize..span.hi as usize - span.lo as usize]
    }
}
