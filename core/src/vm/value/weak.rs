use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak as StdWeak},
};

use super::{HashWeak, Value};

/// A type of weak collection
#[derive(Debug, Clone)]
pub enum Weak {
    /// Represents a JavaScript WeakSet
    Set(WeakSet<RefCell<Value>>),
    /// Represents a JavaScript WeakMap
    Map(WeakMap<RefCell<Value>, RefCell<Value>>),
}

impl Weak {
    /// Returns a reference to the underlying WeakSet, if it is one
    pub fn as_set(&self) -> Option<&WeakSet<RefCell<Value>>> {
        match self {
            Self::Set(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying WeakSet, if it is one
    pub fn as_set_mut(&mut self) -> Option<&mut WeakSet<RefCell<Value>>> {
        match self {
            Self::Set(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a reference to the underlying WeakMap, if it is one
    pub fn as_map(&self) -> Option<&WeakMap<RefCell<Value>, RefCell<Value>>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying WeakMap, if it is one
    pub fn as_map_mut(&mut self) -> Option<&mut WeakMap<RefCell<Value>, RefCell<Value>>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Converts this weak collection to a string
    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::Set(_) => Cow::Borrowed("[object WeakSet]"),
            Self::Map(_) => Cow::Borrowed("[object WeakMap]"),
        }
    }

    /// Inspects this weak collection
    pub fn inspect(&self) -> Cow<str> {
        match self {
            Self::Set(s) => Cow::Owned(format!("WeakSet {{ <{} items> }}", s.0.len())),
            Self::Map(m) => Cow::Owned(format!("WeakMap {{ <{} items> }}", m.0.len())),
        }
    }
}

/// A type that may be either weak or strong
pub enum MaybeWeak<T> {
    /// A weak reference
    Weak(StdWeak<T>),
    /// A strong reference
    Strong(Rc<T>),
}

impl<T> From<StdWeak<T>> for MaybeWeak<T> {
    fn from(w: StdWeak<T>) -> Self {
        Self::Weak(w)
    }
}

impl<T> From<Rc<T>> for MaybeWeak<T> {
    fn from(w: Rc<T>) -> Self {
        Self::Strong(w)
    }
}

impl<T> MaybeWeak<T> {
    /// Returns a [StdWeak] to the underlying MaybeWeak
    pub fn into_weak(self) -> StdWeak<T> {
        match self {
            Self::Weak(w) => w,
            Self::Strong(s) => Rc::downgrade(&s),
        }
    }

    /// Returns a [Rc] to the underlying MaybeWeak
    pub fn into_strong(self) -> Option<Rc<T>> {
        match self {
            Self::Weak(w) => StdWeak::upgrade(&w),
            Self::Strong(s) => Some(s),
        }
    }
}

/// A JavaScript WeakMap
#[derive(Debug, Clone)]
pub struct WeakMap<K, V>(pub HashMap<HashWeak<K>, Rc<V>>);

impl<K, V> WeakMap<K, V> {
    /// Creates a new WeakMap
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Checks whether this WeakMap contains a key
    pub fn has<Q: Into<MaybeWeak<K>>>(&self, key: Q) -> bool {
        let key = key.into();
        let hashweak = HashWeak(key.into_weak());
        self.0.contains_key(&hashweak)
    }

    /// Checks whether this WeakMap contains a key given a value cell
    pub fn has_rc_key(&self, key: &Rc<K>) -> bool {
        let hashweak = HashWeak(Rc::downgrade(key));
        self.0.contains_key(&hashweak)
    }

    /// Looks up a key
    pub fn get<Q: Into<MaybeWeak<K>>>(&self, key: Q) -> Option<&Rc<V>> {
        let key = key.into();
        let hashweak = HashWeak(key.into_weak());
        self.0.get(&hashweak)
    }

    /// Looks up a key, given a value cell
    pub fn get_rc_key(&self, key: &Rc<K>) -> Option<&Rc<V>> {
        let hashweak = HashWeak(Rc::downgrade(key));
        self.0.get(&hashweak)
    }

    /// Adds a value to this WeakMap
    pub fn add<Q: Into<MaybeWeak<K>>>(&mut self, key: Q, value: Rc<V>) {
        let key = key.into();
        let hashweak = HashWeak(key.into_weak());
        self.0.insert(hashweak, value);
    }

    /// Deletes an entry from this WeakMap
    pub fn delete<Q: Into<MaybeWeak<K>>>(&mut self, key: Q) -> bool {
        let key = key.into();
        let hashweak = HashWeak(key.into_weak());
        self.0.remove(&hashweak).is_some()
    }

    /// Deletes an entry from this WeakMap, given a value cell
    pub fn delete_rc_key(&mut self, key: &Rc<K>) -> bool {
        let hashweak = HashWeak(Rc::downgrade(key));
        self.0.remove(&hashweak).is_some()
    }
}

/// A JavaScript WeakSet
#[derive(Debug, Clone)]
pub struct WeakSet<K>(pub HashSet<HashWeak<K>>);

impl<K> WeakSet<K> {
    /// Creates a new WeakSet
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    /// Checks whether this WeakSet contains a key
    pub fn has(&self, rc: &Rc<K>) -> bool {
        let weak = Rc::downgrade(rc);
        let hashweak = HashWeak(weak);
        self.0.contains(&hashweak)
    }

    /// Adds a key to this WeakSet
    pub fn add(&mut self, key: &Rc<K>) {
        let weak = Rc::downgrade(key);
        let hashweak = HashWeak(weak);
        self.0.insert(hashweak);
    }

    /// Deletes a key from this WeakSet
    pub fn delete(&mut self, rc: &Rc<K>) -> bool {
        let weak = Rc::downgrade(rc);
        let hashweak = HashWeak(weak);
        self.0.remove(&hashweak)
    }
}

impl<K> From<HashSet<HashWeak<K>>> for WeakSet<K> {
    fn from(s: HashSet<HashWeak<K>>) -> Self {
        Self(s)
    }
}

impl<K, V> From<HashMap<HashWeak<K>, Rc<V>>> for WeakMap<K, V> {
    fn from(s: HashMap<HashWeak<K>, Rc<V>>) -> Self {
        Self(s)
    }
}
