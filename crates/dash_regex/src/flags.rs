use std::str::FromStr;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    #[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
    pub struct Flags: u8 {
        const GLOBAL = 1;
        const IGNORE_CASE = 2;
    }
}

#[derive(Debug)]
pub enum Error {
    UnknownFlag(char),
}

impl FromStr for Flags {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = Flags::empty();
        for c in s.chars() {
            match c {
                'g' => flags |= Flags::GLOBAL,
                'i' => flags |= Flags::IGNORE_CASE,
                o => return Err(Error::UnknownFlag(o)),
            }
        }
        Ok(flags)
    }
}
