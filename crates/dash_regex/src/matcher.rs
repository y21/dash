use crate::node::Node;
use crate::stream::BorrowedStream;
use crate::visitor::Visit;

pub struct Matcher<'a> {
    nodes: BorrowedStream<'a, Node>,
    text: BorrowedStream<'a, u8>,
}

impl<'a> Matcher<'a> {
    pub fn new(nodes: &'a [Node], text: &'a [u8]) -> Self {
        Self {
            nodes: BorrowedStream::new(nodes),
            text: BorrowedStream::new(text),
        }
    }

    pub fn matches(&mut self) -> bool {
        while !self.nodes.is_eof() {
            if !self.matches_single() {
                return false;
            }
        }
        true
    }

    pub fn matches_single(&mut self) -> bool {
        let node = self.nodes.next().unwrap();
        node.matches(&mut self.text)
    }
}
