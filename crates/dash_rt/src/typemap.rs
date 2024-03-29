use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::ptr;

use dash_proc_macro::Trace;
use dash_vm::gc::trace::Trace;
use nohash::BuildNoHashHasher;

pub trait Key: Any {
    type State: ErasedValue;
}

pub trait ErasedValue: Trace + Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Trace + Any> ErasedValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl dyn ErasedValue {
    fn is<T: 'static>(&self) -> bool {
        self.as_any().is::<T>()
    }

    #[track_caller]
    unsafe fn downcast_unchecked<T: 'static>(&self) -> &T {
        debug_assert!(self.is::<T>());
        &*ptr::from_ref::<dyn ErasedValue>(self).cast::<T>()
    }

    #[track_caller]
    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        debug_assert!(self.is::<T>());
        &mut *ptr::from_mut::<dyn ErasedValue>(self).cast::<T>()
    }
}

#[derive(Default, Trace)]
pub struct TypeMap(HashMap<TypeId, Box<dyn ErasedValue>, BuildNoHashHasher<u64>>);

impl TypeMap {
    pub fn insert<K: Key>(&mut self, k: K, v: K::State) {
        let type_id = k.type_id();
        let value = Box::new(v) as Box<dyn ErasedValue>;
        self.0.insert(type_id, value);
    }

    #[track_caller]
    pub fn get<K: Key>(&self, k: K) -> Option<&K::State> {
        let type_id = k.type_id();

        let value = &**self.0.get(&type_id)?;
        // SAFETY: we only ever insert into the map with K::State
        Some(unsafe { value.downcast_unchecked() })
    }

    #[track_caller]
    pub fn get_mut<K: Key>(&mut self, k: K) -> Option<&mut K::State> {
        let type_id = k.type_id();

        let value = &mut **self.0.get_mut(&type_id)?;
        // SAFETY: we only ever insert into the map with K::State
        Some(unsafe { value.downcast_mut_unchecked() })
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
