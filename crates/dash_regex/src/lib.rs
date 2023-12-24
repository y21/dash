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

pub type Regex = Vec<Node>;

#[cfg(test)]
#[test]
pub fn test() {
    use parser::Parser;

    use crate::matcher::Matcher;

    fn matches(regex: &str, input: &str) -> bool {
        let nodes = Parser::new(regex.as_bytes()).parse_all().unwrap();
        let mut matcher = Matcher::new(&nodes, input.as_bytes());
        matcher.matches().is_some()
    }

    const HEX_REGEX: &str = "^#?([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$";
    assert!(matches(HEX_REGEX, "#aabbccdd"));
    assert!(!matches(HEX_REGEX, "#AAb"));
    assert!(matches(HEX_REGEX, "#aBcDEEf0"));

    assert!(matches("\\d", "a1"));
    assert!(matches("V\\dX", "aV1aVaXaV1Xs"));
    assert!(!matches("V\\dX", "aV1aVaXaV?Xs"));
}
