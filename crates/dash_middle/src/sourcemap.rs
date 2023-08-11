#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub lo: u32,
    pub hi: u32,
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
