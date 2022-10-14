use crate::node::Node;
use crate::stream::BorrowedStream;
use crate::stream::Stream;
use crate::visitor::Visit;

pub struct Matcher<'a> {
    nodes: Stream<Node>,
    text: BorrowedStream<'a, u8>,
}

impl<'a> Matcher<'a> {
    pub fn new(nodes: Vec<Node>, text: &'a [u8]) -> Self {
        Self {
            nodes: Stream::new(nodes),
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

    pub fn nodes_mut(&mut self) -> &mut Stream<Node> {
        &mut self.nodes
    }
}
