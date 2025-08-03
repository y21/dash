use std::marker::PhantomData;

use thin_vec::ThinVec;

use crate::indexvec::{Index, IndexRepr};

#[derive(Debug, Clone)]
pub struct IndexThinVec<T, I>(ThinVec<T>, PhantomData<I>);

impl<T, I: Index> IndexThinVec<T, I> {
    pub fn new() -> Self {
        Self(ThinVec::new(), PhantomData)
    }

    pub fn try_push(&mut self, element: T) -> Option<I> {
        let len = self.0.len();
        let index = I::Repr::from_usize_checked(len)?;
        self.0.push(element);
        Some(I::from_repr(index))
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }
}
impl<T, I: Index> std::ops::Index<I> for IndexThinVec<T, I> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.0[index.into_repr().usize()]
    }
}
impl<T, I> Default for IndexThinVec<T, I> {
    fn default() -> Self {
        Self(ThinVec::default(), PhantomData)
    }
}

#[cfg(feature = "format")]
impl<T: serde::Serialize, I> serde::Serialize for IndexThinVec<T, I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for elem in self.0.iter() {
            seq.serialize_element(elem)?;
        }
        seq.end()
    }
}

#[cfg(feature = "format")]
impl<'de, T: serde::Deserialize<'de>, I> serde::Deserialize<'de> for IndexThinVec<T, I> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Vis<T, I>(PhantomData<(T, I)>);
        impl<'de, T: serde::Deserialize<'de>, I> serde::de::Visitor<'de> for Vis<T, I> {
            type Value = IndexThinVec<T, I>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut data = ThinVec::with_capacity(seq.size_hint().unwrap_or_default());
                while let Some(elem) = seq.next_element::<T>()? {
                    data.push(elem);
                }
                Ok(IndexThinVec(data, PhantomData))
            }
        }
        deserializer.deserialize_seq(Vis(PhantomData))
    }
}
