use std::str::FromStr;

pub use error::Error;
pub use graph::eval::EvalSuccess;

pub mod error;
pub mod flags;
mod node;
mod parser;

mod graph;

pub use flags::Flags;
pub use graph::Regex;
use parser::Parser;

pub trait ParseFlags {
    fn parse(self) -> Result<Flags, Error>;
}

impl ParseFlags for &str {
    fn parse(self) -> Result<Flags, Error> {
        Flags::from_str(self).map_err(Into::into)
    }
}

impl ParseFlags for Flags {
    fn parse(self) -> Result<Flags, Error> {
        Ok(self)
    }
}

pub fn compile(input: &str, flags: impl ParseFlags) -> Result<Regex, Error> {
    let parsed = Parser::new(input.as_bytes()).parse_all()?;
    let flags = flags.parse()?;
    Ok(graph::compile(parsed, flags))
}

#[cfg(test)]
#[test]
pub fn test() {
    fn assert_matches_groups(regex: &Regex, input: &str, groups: &[&str]) {
        let res = regex.eval(input).unwrap();

        for (&expected, got) in groups.iter().zip(&res.groups[1..]) {
            let (from, to, _) = got.expect("no group");
            assert_eq!(expected, &input[from as usize..to as usize]);
        }
    }

    let hex_regex = compile(
        "^#?([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$",
        "",
    )
    .unwrap();
    assert!(hex_regex.matches("#aabbccdd"));
    assert!(!hex_regex.matches("#AAb"));
    assert!(hex_regex.matches("#aBcDEEf0"));

    assert!(compile("\\d", "").unwrap().matches("a1"));
    assert!(compile("V\\dX", "").unwrap().matches("aV1aVaXaV1Xs"));
    assert!(!compile("V\\dX", "").unwrap().matches("aV1aVaXaV?Xs"));

    let rgb_regex = compile(r"rgb[\s|\(]+((?:[-\+]?\d*\.\d+%?)|(?:[-\+]?\d+%?))[,|\s]+((?:[-\+]?\d*\.\d+%?)|(?:[-\+]?\d+%?))[,|\s]+((?:[-\+]?\d*\.\d+%?)|(?:[-\+]?\d+%?))\s*\)?","").unwrap();
    assert!(rgb_regex.matches("rgb(255, 255, 255)"));
    assert_matches_groups(&rgb_regex, "rgb(144, 17, 9)", &["144", "17", "9"]);

    // Backtracking
    assert_matches_groups(&compile("x(.+)x", "").unwrap(), "vxxxv", &["x"]);
    assert_matches_groups(&compile(".(.)+abcd", "").unwrap(), "vxabcdabcabcabcabc", &["x"]);
    assert_matches_groups(&compile("(.+)+a", "").unwrap(), "ba", &["b"]);
    // Degenerate backtracking
    assert_matches_groups(&compile("(.+)+ac", "").unwrap(), "bacbaabaabaabaa", &["b"]);

    assert_matches_groups(&compile("(ab+){3,}", "").unwrap(), "ababab", &["ab"]);
    assert_matches_groups(&compile("(([ab]+)b){3,}", "").unwrap(), "abababaa", &["ab", "a"]);
    assert!(compile("(([ab]+)b){3,}", "").unwrap().eval("ababaaaa").is_err());
    assert!(compile("(([ab]+)b){3,}", "").unwrap().eval("ababaaba").is_ok());

    // Infinite regex needs to terminate eventually
    assert_matches_groups(&compile("(.?)+", "").unwrap(), "", &[""]);
    assert_matches_groups(&compile("(.?)+", "").unwrap(), "aa", &["a"]);

    // ^ anchor must not match when retrying substrings
    assert!(!compile("^m", "").unwrap().matches("ama"));
    assert!(compile("^m", "").unwrap().matches("ma"));
}
