pub struct BorrowedStream<'a, T> {
    source: &'a [T],
    index: usize,
}

impl<'a, T> BorrowedStream<'a, T> {
    pub fn new(source: &'a [T]) -> Self {
        Self { index: 0, source }
    }

    pub fn next(&mut self) -> Option<&'a T> {
        let data = self.source.get(self.index);
        self.index += 1;
        data
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    pub fn current(&self) -> Option<&T> {
        self.source.get(self.index)
    }

    pub fn is_eof(&self) -> bool {
        self.index >= self.source.len()
    }
}
