use crate::matcher::Groups;
use crate::node::{Anchor, CharacterClassItem, GroupCaptureMode, MetaSequence, Node};
use crate::stream::BorrowedStream;

pub trait Visit<'a> {
    fn matches(&self, s: &mut BorrowedStream<'a, u8>, groups: &mut Groups) -> bool;
}

impl<'a> Visit<'a> for Anchor {
    fn matches(&self, s: &mut BorrowedStream<'a, u8>, _: &mut Groups) -> bool {
        match self {
            Anchor::StartOfString => s.index() == 0,
            Anchor::EndOfString => s.is_eof(),
        }
    }
}

impl<'a> Visit<'a> for MetaSequence {
    fn matches(&self, s: &mut BorrowedStream<'a, u8>, _: &mut Groups) -> bool {
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
    fn matches(&self, s: &mut BorrowedStream<'a, u8>, groups: &mut Groups) -> bool {
        match self {
            Node::LiteralCharacter(lit) => {
                let matches = s.current().map(|c| c == lit).unwrap_or(false);
                if matches {
                    s.advance();
                }
                matches
            }
            Node::Optional(node) => {
                node.matches(s, groups);
                true
            }
            Node::Group(capture, group) => {
                let before = s.index();
                let all_matched = group.iter().all(|node| node.matches(s, groups));

                match capture {
                    GroupCaptureMode::Id(id) if all_matched => {
                        groups.set(*id, before..s.index());
                        true
                    }
                    _ => all_matched,
                }
            }
            Node::Or(left, right) => {
                let left_index = s.index();
                let left_matches = left.iter().all(|node| node.matches(s, groups));
                if left_matches {
                    return true;
                }
                s.set_index(left_index);
                right.iter().all(|node| node.matches(s, groups))
            }
            Node::Anchor(anchor) => anchor.matches(s, groups),
            Node::MetaSequence(seq) => seq.matches(s, groups),
            Node::Repetition { node, min, max } => {
                let mut count = 0;
                while !s.is_eof() {
                    if let Some(max) = max {
                        if count >= *max {
                            break;
                        }
                    }

                    if !node.matches(s, groups) {
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
                    CharacterClassItem::Node(ref node) => node.matches(s, groups),
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
