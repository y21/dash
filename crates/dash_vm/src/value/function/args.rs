use std::ops::Deref;

use smallvec::SmallVec;

use crate::value::Value;

pub type SmallArgsRepr = SmallVec<[Value; 3]>;

#[derive(Default, Clone, Debug)]
pub struct CallArgs(SmallArgsRepr);

impl CallArgs {
    pub fn empty() -> Self {
        Self(SmallArgsRepr::new())
    }
}

impl<const N: usize> From<[Value; N]> for CallArgs {
    fn from(value: [Value; N]) -> Self {
        CallArgs(SmallVec::from_slice(&value))
    }
}

impl From<&[Value]> for CallArgs {
    fn from(value: &[Value]) -> Self {
        CallArgs(SmallVec::from_slice(value))
    }
}

impl From<SmallArgsRepr> for CallArgs {
    fn from(v: SmallArgsRepr) -> Self {
        Self(v)
    }
}

impl From<Vec<Value>> for CallArgs {
    fn from(v: Vec<Value>) -> Self {
        Self(SmallVec::from_vec(v))
    }
}

impl Extend<Value> for CallArgs {
    fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl FromIterator<Value> for CallArgs {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Self(SmallVec::from_iter(iter))
    }
}

impl IntoIterator for CallArgs {
    type Item = Value;

    type IntoIter = smallvec::IntoIter<[Value; 3]>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a CallArgs {
    type Item = &'a Value;

    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for CallArgs {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
