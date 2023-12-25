use std::borrow::Borrow;
use std::cell::Cell;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::rc::Rc;

use hashbrown::hash_map::EntryRef;
use rustc_hash::{FxHashMap, FxHasher};

fn fxhash(s: &str) -> u64 {
    let mut hasher = FxHasher::default();
    s.hash(&mut hasher);
    hasher.finish()
}

pub struct StringData {
    string: Rc<str>,
}

pub struct StringInterner {
    storage: Vec<Option<StringData>>,
    mapping: hashbrown::HashMap<Rc<str>, Symbol, BuildHasherDefault<FxHasher>>,
    /// List of free indices in the storage
    free: Vec<RawSymbol>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            mapping: hashbrown::HashMap::default(),
            free: Vec::new(),
        }
    }

    pub fn intern(&mut self, string: impl Borrow<str>) -> Symbol {
        let string = string.borrow();
        let hash = fxhash(string);
        match self.mapping.entry_ref(string) {
            EntryRef::Occupied(occ) => occ.get().clone(),
            EntryRef::Vacant(vac) => {
                if let Some(id) = self.free.pop() {
                    self.storage[id as usize] = Some(StringData {
                        string: Rc::from(string),
                    });
                    vac.insert(Symbol(id));
                    Symbol(id)
                } else {
                    let id: RawSymbol = self.storage.len().try_into().expect("too many strings");
                    let string = Rc::from(string);
                    self.storage.push(Some(StringData { string }));
                    vac.insert(Symbol(id));
                    Symbol(id)
                }
            }
        }
    }

    // pub fn remove(&mut self, symbol: Symbol) {
    //     self.storage[symbol.0 as usize] = None;
    //     self.free.push(symbol.0);
    // }

    pub fn resolve(&mut self, symbol: Symbol) -> &str {
        &self.storage[symbol.0 as usize]
            .as_ref()
            .expect("tombstone symbol")
            .string
    }
}

type RawSymbol = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(RawSymbol);

impl Symbol {}
