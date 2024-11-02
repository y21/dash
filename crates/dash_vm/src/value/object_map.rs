use std::hash::BuildHasherDefault;
use std::mem;

use rustc_hash::FxHasher;

use crate::gc::trace::Trace;

use super::object::{PropertyKey, PropertyValue};

const MAX_SEQUENTIAL: usize = 16;

type HashedValues = hashbrown::HashMap<PropertyKey, PropertyValue, BuildHasherDefault<FxHasher>>;

#[derive(Debug, Clone)]
enum InnerValues {
    Small(Vec<(PropertyKey, PropertyValue)>),
    Large(HashedValues),
}

#[derive(Debug, Clone)]
pub struct ObjectMap(InnerValues);

unsafe impl Trace for ObjectMap {
    fn trace(&self, cx: &mut crate::gc::trace::TraceCtxt<'_>) {
        let Self(values) = self;
        match values {
            InnerValues::Small(vec) => vec.as_slice().trace(cx),
            InnerValues::Large(hash_map) => hash_map.trace(cx),
        }
    }
}

impl Default for ObjectMap {
    fn default() -> Self {
        Self(InnerValues::Small(Vec::new()))
    }
}

impl ObjectMap {
    fn convert_to_hashed(&mut self) -> &mut HashedValues {
        let old = mem::replace(&mut self.0, InnerValues::Small(Vec::new()));

        match old {
            InnerValues::Small(vec) => {
                self.0 = InnerValues::Large(vec.into_iter().collect());
            }
            InnerValues::Large(hash_map) => {
                self.0 = InnerValues::Large(hash_map);
            }
        }

        match &mut self.0 {
            InnerValues::Small(_) => unreachable!(),
            InnerValues::Large(hash_map) => hash_map,
        }
    }

    fn reserve(&mut self, additional: usize) {
        match &mut self.0 {
            InnerValues::Small(sm) => {
                if sm.len() + additional > MAX_SEQUENTIAL {
                    self.convert_to_hashed().reserve(additional);
                } else {
                    sm.reserve(additional);
                }
            }
            InnerValues::Large(large) => {
                large.reserve(additional);
            }
        }
    }

    pub fn insert(&mut self, key: PropertyKey, value: PropertyValue) {
        self.reserve(1);
        match &mut self.0 {
            InnerValues::Small(vec) => {
                if !vec.iter().any(|(k, _)| *k == key) {
                    vec.push((key, value));
                }
            }
            InnerValues::Large(hash_map) => drop(hash_map.insert(key, value)),
        }
    }

    pub fn remove(&mut self, key: PropertyKey) -> Option<PropertyValue> {
        match &mut self.0 {
            InnerValues::Small(vec) => {
                let remove_idx = vec.iter().position(|(k, _)| *k == key)?;
                Some(vec.swap_remove(remove_idx).1)
            }
            InnerValues::Large(hash_map) => hash_map.remove(&key),
        }
    }

    pub fn get(&self, key: PropertyKey) -> Option<&PropertyValue> {
        match &self.0 {
            InnerValues::Small(vec) => vec.iter().find_map(|(k, v)| (*k == key).then_some(v)),
            InnerValues::Large(hash_map) => hash_map.get(&key),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = PropertyKey> + use<'_> {
        enum KeysIter<'a> {
            Small(std::slice::Iter<'a, (PropertyKey, PropertyValue)>),
            Large(hashbrown::hash_map::Keys<'a, PropertyKey, PropertyValue>),
        }
        impl Iterator for KeysIter<'_> {
            type Item = PropertyKey;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    KeysIter::Small(iter) => iter.next().map(|(k, _)| *k),
                    KeysIter::Large(keys) => keys.next().copied(),
                }
            }
        }

        match &self.0 {
            InnerValues::Small(vec) => KeysIter::Small(vec.iter()),
            InnerValues::Large(hash_map) => KeysIter::Large(hash_map.keys()),
        }
    }

    pub fn entry(&mut self, key: PropertyKey) -> Entry<'_> {
        match &mut self.0 {
            InnerValues::Small(vec) => {
                let index = vec.iter().position(|(k, _)| *k == key);
                Entry::Small(vec, key, index)
            }
            InnerValues::Large(hash_map) => Entry::Large(hash_map.entry(key)),
        }
    }
}

impl FromIterator<(PropertyKey, PropertyValue)> for ObjectMap {
    fn from_iter<T: IntoIterator<Item = (PropertyKey, PropertyValue)>>(it: T) -> Self {
        let it = it.into_iter();
        let mut this = Self::default();
        this.reserve(it.size_hint().0);
        for (k, v) in it {
            this.insert(k, v);
        }
        this
    }
}

pub enum Entry<'a> {
    Small(&'a mut Vec<(PropertyKey, PropertyValue)>, PropertyKey, Option<usize>),
    Large(hashbrown::hash_map::Entry<'a, PropertyKey, PropertyValue, BuildHasherDefault<FxHasher>>),
}

impl Entry<'_> {
    pub fn modify_or_insert(self, modify: impl FnOnce(&mut PropertyValue), insert: impl FnOnce() -> PropertyValue) {
        match self {
            Entry::Small(vec, _, Some(index)) => modify(&mut vec[index].1),
            Entry::Small(vec, key, None) => vec.push((key, insert())),
            Entry::Large(hashbrown::hash_map::Entry::Occupied(mut occ)) => modify(occ.get_mut()),
            Entry::Large(hashbrown::hash_map::Entry::Vacant(vac)) => drop(vac.insert(insert())),
        }
    }
}
