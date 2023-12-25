use std::ops::Range;

use smallvec::{smallvec, SmallVec};

use crate::node::Node;
use crate::parser::ParsedRegex;
use crate::stream::BorrowedStream;
use crate::visitor::Visit;

pub struct Matcher<'a> {
    nodes: BorrowedStream<'a, Node>,
    text: BorrowedStream<'a, u8>,
    pub groups: Groups,
}

#[derive(Debug, Clone)]
pub struct Groups(SmallVec<[Option<Range<usize>>; 1]>);

impl Groups {
    pub fn new(count: usize) -> Self {
        Self(smallvec![None; count])
    }

    pub fn set(&mut self, index: usize, range: Range<usize>) {
        self.0[index] = Some(range);
    }

    pub fn get(&mut self, index: usize) -> Option<Range<usize>> {
        self.0[index].clone()
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<Range<usize>>> + '_ {
        self.0.iter().cloned()
    }
}

impl<'a> Matcher<'a> {
    pub fn new(parsed_regex: &'a ParsedRegex, text: &'a [u8]) -> Self {
        Self {
            nodes: BorrowedStream::new(parsed_regex.nodes.as_slice()),
            text: BorrowedStream::new(text),
            groups: Groups::new(parsed_regex.group_count),
        }
    }

    pub fn matches(&mut self) -> bool {
        let mut index = self.text.index();

        // TODO: what if text.len() == 0?

        while index < self.text.len() {
            if self.nodes.is_eof() {
                // all regex nodes matched
                self.groups.set(0, index..self.text.index());
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
        node.matches(&mut self.text, &mut self.groups)
    }
}
