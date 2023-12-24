use crate::node::{Anchor, CharacterClassItem, MetaSequence, Node};
use crate::stream::BorrowedStream;

pub trait Visit<'a> {
    fn matches(&self, s: &mut BorrowedStream<'a, u8>) -> bool;
}

impl<'a> Visit<'a> for Anchor {
    fn matches(&self, s: &mut BorrowedStream<'a, u8>) -> bool {
        match self {
            Anchor::StartOfString => s.index() == 0,
            Anchor::EndOfString => s.is_eof(),
        }
    }
}

impl<'a> Visit<'a> for MetaSequence {
    fn matches(&self, s: &mut BorrowedStream<'a, u8>) -> bool {
        match self {
            Self::Digit => {
                let is_digit = s.current().map(|c| c.is_ascii_digit()).unwrap_or(false);
                if is_digit {
                    s.advance();
                }
                is_digit
            }
            Self::Word => {
                let matches = s
                    .current()
                    .map(|c| c.is_ascii_alphanumeric() || *c == b'_')
                    .unwrap_or(false);
                if matches {
                    s.advance();
                }
                matches
            }
            Self::Whitespace => {
                let matches = s.current().map(|c| c.is_ascii_whitespace()).unwrap_or(false);
                if matches {
                    s.advance();
                }
                matches
            }
        }
    }
}

impl<'a> Visit<'a> for Node {
    fn matches(&self, s: &mut BorrowedStream<'a, u8>) -> bool {
        match self {
            Node::LiteralCharacter(lit) => {
                let matches = s.current().map(|c| c == lit).unwrap_or(false);
                if matches {
                    s.advance();
                }
                matches
            }
            Node::Optional(node) => {
                node.matches(s);
                true
            }
            Node::Group(_, group) => group.iter().all(|node| node.matches(s)),
            Node::Or(left, right) => {
                let left_index = s.index();
                let left_matches = left.iter().all(|node| node.matches(s));
                if left_matches {
                    return true;
                }
                s.set_index(left_index);
                right.iter().all(|node| node.matches(s))
            }
            Node::Anchor(anchor) => anchor.matches(s),
            Node::MetaSequence(seq) => seq.matches(s),
            Node::Repetition { node, min, max } => {
                let mut count = 0;
                while !s.is_eof() {
                    if let Some(max) = max {
                        if count >= *max {
                            break;
                        }
                    }

                    if !node.matches(s) {
                        break;
                    }
                    count += 1;
                }
                if count < *min {
                    return false;
                }
                true
            }
            Node::AnyCharacter => {
                s.next();
                true
            }
            Node::CharacterClass(nodes) => {
                let Some(&cur) = s.current() else { return false };

                nodes.iter().any(|node| match *node {
                    CharacterClassItem::Node(ref node) => node.matches(s),
                    CharacterClassItem::Range(start, end) => {
                        let matches = (start..=end).contains(&cur);
                        if matches {
                            s.advance();
                        }
                        matches
                    }
                })
            }
        }
    }
}
