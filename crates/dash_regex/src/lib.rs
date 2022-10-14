pub mod error;
pub mod matcher;
pub mod node;
pub mod parser;
mod stream;
mod visitor;

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

    dbg!(matches("a[\\db]{2,}", "ab342cc"));
}
