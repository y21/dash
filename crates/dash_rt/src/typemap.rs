use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::ptr;

use nohash::BuildNoHashHasher;

pub trait Key: Any {
    type State: 'static;
}

#[derive(Default)]
pub struct TypeMap(HashMap<TypeId, Box<dyn Any>, BuildNoHashHasher<u64>>);

unsafe fn downcast_mut_unchecked<T: 'static>(v: &mut dyn Any) -> &mut T {
    debug_assert!(v.is::<T>());
    &mut *ptr::from_mut::<dyn Any>(v).cast::<T>()
}

unsafe fn downcast_unchecked<T: 'static>(v: &dyn Any) -> &T {
    debug_assert!(v.is::<T>());
    &*ptr::from_ref::<dyn Any>(v).cast::<T>()
}

impl TypeMap {
    pub fn insert<K: Key>(&mut self, k: K, v: K::State) {
        let type_id = k.type_id();
        self.0.insert(type_id, Box::<K::State>::new(v));
    }

    pub fn get<K: Key>(&self, k: K) -> Option<&K::State> {
        let type_id = k.type_id();

        let value = self.0.get(&type_id)?;
        // SAFETY: we only ever insert into the map with K::State
        Some(unsafe { downcast_unchecked(&**value) })
    }

    pub fn get_mut<K: Key>(&mut self, k: K) -> Option<&mut K::State> {
        let type_id = k.type_id();

        let value = self.0.get_mut(&type_id)?;
        // SAFETY: we only ever insert into the map in the branch below
        Some(unsafe { downcast_mut_unchecked(&mut **value) })
    }
}

impl<K: Key> Index<K> for TypeMap {
    type Output = K::State;

    fn index(&self, index: K) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<K: Key> IndexMut<K> for TypeMap {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::typemap::Key;

    use super::TypeMap;

    #[test]
    fn test() {
        struct FsModule;
        impl Key for FsModule {
            type State = Vec<&'static str>;
        }
        struct OtherModule;
        impl Key for OtherModule {
            type State = ();
        }

        let mut map = TypeMap::default();
        map.insert(FsModule, vec!["a", "b"]);
        map.insert(OtherModule, ());
        assert_eq!(&map[FsModule], &["a", "b"]);
        assert_eq!(&map[OtherModule], &());
    }
}
