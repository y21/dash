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
        let mut index = self.text.index();

        while index < self.text.len() {
            if self.nodes.is_eof() {
                // all regex nodes matched
                return true;
            }

            if !self.matches_single() {
                index += 1;
                self.nodes.set_index(0);
                self.text.set_index(index);
            }
        }

        false
    }

    pub fn matches_single(&mut self) -> bool {
        let node = self.nodes.next().unwrap();
        node.matches(&mut self.text)
    }
}
