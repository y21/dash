use std::marker::PhantomData;

use thin_vec::ThinVec;

pub trait Index: Copy + TryFrom<usize> + Into<usize> {}

#[derive(Debug, Clone)]
pub struct IndexThinVec<T, I>(ThinVec<T>, PhantomData<I>);

impl<T, I: Index> IndexThinVec<T, I> {
    pub fn new() -> Self {
        Self(ThinVec::new(), PhantomData)
    }

    pub fn try_push(&mut self, element: T) -> Result<I, <I as TryFrom<usize>>::Error> {
        let len = self.0.len();
        self.0.push(element);
        I::try_from(len)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
}
impl<T, I: Index> std::ops::Index<I> for IndexThinVec<T, I> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.0[Into::<usize>::into(index)]
    }
}
impl<T, I> Default for IndexThinVec<T, I> {
    fn default() -> Self {
        Self(ThinVec::default(), PhantomData)
    }
}

#[macro_export]
macro_rules! index_type {
    ($name:ident $repr:ty) => {
        #[derive(Copy, Clone, Debug)]
        pub struct $name(pub $repr);

        impl TryFrom<usize> for $name {
            type Error = <$repr as TryFrom<usize>>::Error;

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                Ok(Self(<$repr>::try_from(value)?))
            }
        }
        impl From<$name> for usize {
            fn from(v: $name) -> usize {
                v.0.into()
            }
        }

        impl $crate::indexvec::Index for $name {}
    };
}

impl Index for u16 {}

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
