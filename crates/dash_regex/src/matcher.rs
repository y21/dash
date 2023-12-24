use std::ops::Range;

use smallvec::{smallvec, SmallVec};

use crate::node::Node;
use crate::stream::BorrowedStream;
use crate::visitor::Visit;

pub struct Matcher<'a> {
    nodes: BorrowedStream<'a, Node>,
    text: BorrowedStream<'a, u8>,
}

#[derive(Debug)]
pub struct Match {
    pub groups: SmallVec<[Range<usize>; 1]>,
}

impl<'a> Matcher<'a> {
    pub fn new(nodes: &'a [Node], text: &'a [u8]) -> Self {
        Self {
            nodes: BorrowedStream::new(nodes),
            text: BorrowedStream::new(text),
        }
    }

    pub fn matches(&mut self) -> Option<Match> {
        let mut index = self.text.index();

        // TODO: what if text.len() == 0?

        while index < self.text.len() {
            if self.nodes.is_eof() {
                // all regex nodes matched
                return Some(Match {
                    groups: smallvec![index..self.text.index()],
                });
            }

            if !self.matches_single() {
                index += 1;
                self.nodes.set_index(0);
                self.text.set_index(index);
            }
        }

        None
    }

    pub fn matches_single(&mut self) -> bool {
        let node = self.nodes.next().unwrap();
        node.matches(&mut self.text)
    }
}
