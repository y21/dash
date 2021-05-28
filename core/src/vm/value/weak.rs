use std::{
    collections::{HashMap, HashSet},
    rc::{Rc, Weak as StdWeak},
};

use super::HashWeak;

pub enum MaybeWeak<T> {
    Weak(StdWeak<T>),
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
    pub fn into_weak(self) -> StdWeak<T> {
        match self {
            Self::Weak(w) => w,
            Self::Strong(s) => Rc::downgrade(&s),
        }
    }

    pub fn into_strong(self) -> Option<Rc<T>> {
        match self {
            Self::Weak(w) => StdWeak::upgrade(&w),
            Self::Strong(s) => Some(s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeakMap<K, V>(pub HashMap<HashWeak<K>, Rc<V>>);

impl<K, V> WeakMap<K, V> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn has<Q: Into<MaybeWeak<K>>>(&self, key: Q) -> bool {
        let key = key.into();
        let hashweak = HashWeak(key.into_weak());
        self.0.contains_key(&hashweak)
    }

    pub fn has_rc_key(&self, key: &Rc<K>) -> bool {
        let hashweak = HashWeak(Rc::downgrade(key));
        self.0.contains_key(&hashweak)
    }

    pub fn get<Q: Into<MaybeWeak<K>>>(&self, key: Q) -> Option<&Rc<V>> {
        let key = key.into();
        let hashweak = HashWeak(key.into_weak());
        self.0.get(&hashweak)
    }

    pub fn get_rc_key(&self, key: &Rc<K>) -> Option<&Rc<V>> {
        let hashweak = HashWeak(Rc::downgrade(key));
        self.0.get(&hashweak)
    }

    pub fn add<Q: Into<MaybeWeak<K>>>(&mut self, key: Q, value: Rc<V>) {
        let key = key.into();
        let hashweak = HashWeak(key.into_weak());
        self.0.insert(hashweak, value);
    }

    pub fn delete<Q: Into<MaybeWeak<K>>>(&mut self, key: Q) -> bool {
        let key = key.into();
        let hashweak = HashWeak(key.into_weak());
        self.0.remove(&hashweak).is_some()
    }

    pub fn delete_rc_key(&mut self, key: &Rc<K>) -> bool {
        let hashweak = HashWeak(Rc::downgrade(key));
        self.0.remove(&hashweak).is_some()
    }
}

#[derive(Debug, Clone)]
pub struct WeakSet<K>(pub HashSet<HashWeak<K>>);

impl<K> WeakSet<K> {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn has(&self, rc: &Rc<K>) -> bool {
        let weak = Rc::downgrade(rc);
        let hashweak = HashWeak(weak);
        self.0.contains(&hashweak)
    }

    pub fn add(&mut self, key: &Rc<K>) {
        let weak = Rc::downgrade(key);
        let hashweak = HashWeak(weak);
        self.0.insert(hashweak);
    }

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
