use std::ops::{BitOr, BitOrAssign};

#[derive(Debug, Copy, Clone)]
pub enum JoinChange {
    Unchanged,
    Changed,
}

impl BitOr for JoinChange {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (JoinChange::Unchanged, JoinChange::Unchanged) => JoinChange::Unchanged,
            _ => JoinChange::Changed,
        }
    }
}
impl BitOrAssign for JoinChange {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

pub trait Join: Sized {
    fn join(&self, other: &Self) -> (Self, JoinChange);
}

pub fn join_slice<T: Join + Clone, C: FromIterator<T>>(a: &[T], b: &[T]) -> (C, JoinChange) {
    let mut change = JoinChange::Unchanged;
    let collection = a
        .iter()
        .zip(b.iter())
        .map(|(a, b)| {
            let (out, ch) = a.join(b);
            change |= ch;
            out
        })
        .collect();

    (collection, change)
}
