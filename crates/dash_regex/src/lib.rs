pub use error::Error;
pub use matcher::Matcher;
pub use node::Node;
pub use parser::Parser;

pub mod error;
pub mod matcher;
pub mod node;
pub mod parser;
mod stream;
mod visitor;

pub use parser::ParsedRegex;

#[cfg(test)]
#[test]
pub fn test() {
    use parser::Parser;

    use crate::matcher::Matcher;

    fn matches(regex: &str, input: &str) -> bool {
        let nodes = Parser::new(regex.as_bytes()).parse_all().unwrap();
        let mut matcher = Matcher::new(&nodes, input.as_bytes());
        matcher.matches()
    }

    fn matches_groups(regex: &str, input: &str, groups: &[&str]) -> bool {
        let nodes = Parser::new(regex.as_bytes()).parse_all().unwrap();
        let mut matcher = Matcher::new(&nodes, input.as_bytes());
        matcher.matches()
            && nodes.group_count - 1 == groups.len()
            && matcher
                .groups
                .iter()
                .skip(1)
                .zip(groups)
                .all(|(group, expected)| group.map(|range| &input[range]) == Some(*expected))
    }

    const HEX_REGEX: &str = "^#?([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$";
    assert!(matches(HEX_REGEX, "#aabbccdd"));
    assert!(!matches(HEX_REGEX, "#AAb"));
    assert!(matches(HEX_REGEX, "#aBcDEEf0"));

    assert!(matches("\\d", "a1"));
    assert!(matches("V\\dX", "aV1aVaXaV1Xs"));
    assert!(!matches("V\\dX", "aV1aVaXaV?Xs"));

    const RGB: &str = r"rgb[\s|\(]+((?:[-\+]?\d*\.\d+%?)|(?:[-\+]?\d+%?))[,|\s]+((?:[-\+]?\d*\.\d+%?)|(?:[-\+]?\d+%?))[,|\s]+((?:[-\+]?\d*\.\d+%?)|(?:[-\+]?\d+%?))\s*\)?";
    assert!(matches(RGB, "rgb(255, 255, 255)"));
    assert!(matches_groups(RGB, "rgb(144, 17, 9)", &["144", "17", "9"]));
}
