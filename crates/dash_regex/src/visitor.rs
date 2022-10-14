use crate::node::MetaSequence;
use crate::node::Node;
use crate::stream::BorrowedStream;

pub trait Visit<'a> {
    fn matches(&self, s: &mut BorrowedStream<'a, u8>) -> bool;
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
                let mut matches = false;
                for node in nodes {
                    if node.matches(s) {
                        matches = true;
                        break;
                    }
                }
                matches
            }
        }
    }
}
