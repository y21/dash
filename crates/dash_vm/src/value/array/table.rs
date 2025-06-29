use std::mem;

use dash_proc_macro::Trace;
use rustc_hash::FxHashMap;

use crate::value::object::PropertyValue;

use super::MaybeHoley;

/// A more general fallback implementation for arrays that supports holes
#[derive(Debug, Trace, Default)]
pub struct ArrayTable {
    /// The length. This is the highest index + 1
    len: u32,
    map: FxHashMap<u32, PropertyValue>,
}

impl ArrayTable {
    pub fn new() -> Self {
        Self {
            len: 0,
            map: FxHashMap::default(),
        }
    }

    pub fn has_holes(&self) -> bool {
        self.len as usize != self.map.len()
    }

    pub fn take_into_sorted_array(&mut self) -> Vec<PropertyValue> {
        assert!(!self.has_holes());

        let mut array = mem::take(&mut self.map).into_iter().collect::<Vec<_>>();
        array.sort_unstable_by_key(|&(k, _)| k);
        array.into_iter().map(|(_, v)| v).collect()
    }

    pub fn with_len(len: u32) -> Self {
        Self {
            len,
            map: FxHashMap::default(),
        }
    }

    pub fn from_iter(iter: impl IntoIterator<Item = PropertyValue>, len: u32) -> Self {
        Self {
            len,
            map: iter.into_iter().enumerate().map(|(i, p)| (i as u32, p)).collect(),
        }
    }

    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn resize(&mut self, to: u32) {
        if to < self.len {
            // Truncate
            self.map.retain(|&k, _| k < to);
        }
        self.len = to;
    }

    pub fn get(&self, index: u32) -> Option<MaybeHoley<PropertyValue>> {
        match self.map.get(&index).copied() {
            Some(v) => Some(MaybeHoley::Some(v)),
            None => {
                if index < self.len {
                    Some(MaybeHoley::Hole)
                } else {
                    None
                }
            }
        }
    }

    pub fn set(&mut self, index: u32, value: PropertyValue) {
        if index >= self.len {
            self.len = index + 1;
        }
        self.map.insert(index, value);
    }

    pub fn push(&mut self, value: PropertyValue) {
        self.map.insert(self.len, value);
        self.len += 1;
    }

    /// Removes an element by marking it as a hole. Notably, it does not do any truncating.
    pub fn delete_make_hole(&mut self, index: u32) -> Option<MaybeHoley<PropertyValue>> {
        match self.map.remove(&index) {
            Some(v) => Some(MaybeHoley::Some(v)),
            None => {
                if index < self.len {
                    Some(MaybeHoley::Hole)
                } else {
                    None
                }
            }
        }
    }
}
