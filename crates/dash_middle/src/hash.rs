//! Hashing related utility types and functions.
//! This is used both by the Vm for storing objects,
//! as well as precomputing hashes.
//! They must use the same hashing algorithm to be compatible,
//! which is why we want to closely couple them.

use std::borrow::Cow;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::hash::Hasher;

use hashbrown::HashMap;

/// Never call [`ObjectMap::default`]! Use [`build_object_map`]!
///
/// If you do see some code that is doing this, please open an issue!
pub type ObjectMap<K, V> = HashMap<K, V, BuildHasherDefault<ahash::AHasher>>;

/// Computes the hash of a value.
/// This uses the same hashing algorithm as for what the Vm expects,
/// so this can be used to precompute the hash at compile time.
fn hash<T: Hash>(v: T) -> u64 {
    let mut h = ahash::AHasher::default();
    v.hash(&mut h);
    h.finish()
}

/// Computes the hash of a static property key.
pub fn hash_property_key(key: &str) -> u64 {
    // This enum "emulates" the real `PropertyKey` enum in the dash_vm crate (can't use that here in the dash_middle crate).
    #[derive(Hash)]
    enum PropertyKey<'a> {
        Static(Cow<'a, str>),
        #[allow(dead_code)]
        Dummy(()),
    }
    hash(PropertyKey::Static(Cow::Borrowed(key)))
}

/// This is the function you should use for building an object map.
/// This is important, since other ways of creating the HashMap
/// might use a different hashing algorithm (or, even if you do use
/// the same hasher, the keys might be different/RNG might be different),
/// which would introduce very strange behavior and fun debugging sessions.
pub fn build_object_map<K, V>() -> ObjectMap<K, V> {
    HashMap::with_hasher(BuildHasherDefault::<ahash::AHasher>::default())
}
