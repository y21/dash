use crate::gc::trace::Trace;

use super::MaybeHoley;

/// An array type that can have holes.
//
// INTERNAL INVARIANTS:
//  - there should never be zero-sized holes
//  - there should be no consecutive holes; they should be represented as one
//      - for example, instead of [Hole(2), Hole(2)], it should be [Hole(4)]
#[derive(Debug)]
pub struct HoleyArray<T>(Vec<Element<T>>);

unsafe impl<T: Trace> Trace for HoleyArray<T> {
    fn trace(&self, cx: &mut crate::gc::trace::TraceCtxt<'_>) {
        for v in &self.0 {
            match v {
                Element::Value(v) => v.trace(cx),
                Element::Hole { count: _ } => {}
            }
        }
    }
}
impl<T> Default for HoleyArray<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Element<T> {
    Value(T),
    Hole { count: usize },
}
impl<T> Element<T> {
    pub fn elements(&self) -> usize {
        match *self {
            Self::Value(_) => 1,
            Self::Hole { count } => count,
        }
    }
}

impl<T> From<Vec<Element<T>>> for HoleyArray<T> {
    fn from(value: Vec<Element<T>>) -> Self {
        Self(value)
    }
}

struct Lookup {
    chunk_start: usize,
    chunk_count: usize,
    holey_index: usize,
}

impl<T> HoleyArray<T> {
    pub fn into_inner(self) -> Vec<Element<T>> {
        self.0
    }

    pub fn inner(&self) -> &[Element<T>] {
        &self.0
    }

    /// Checks if there are any holes in this array
    pub fn has_hole(&self) -> bool {
        self.0.iter().any(|e| matches!(e, Element::Hole { .. }))
    }

    pub fn compute_len(&self) -> usize {
        self.0.iter().map(Element::elements).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// "Looks up" an index
    fn lookup_index(&self, at: usize) -> Option<Lookup> {
        let mut norm = 0;
        for (index, element) in self.0.iter().enumerate() {
            match *element {
                Element::Value(_) => {
                    if norm == at {
                        return Some(Lookup {
                            chunk_start: norm,
                            chunk_count: 1,
                            holey_index: index,
                        });
                    } else {
                        norm += 1;
                    }
                }
                Element::Hole { count } => {
                    let range = norm..norm + count;
                    if range.contains(&at) {
                        return Some(Lookup {
                            chunk_start: norm,
                            chunk_count: count,
                            holey_index: index,
                        });
                    } else {
                        norm += count;
                    }
                }
            }
        }
        None
    }

    pub fn get(&self, at: usize) -> Option<MaybeHoley<&T>> {
        self.lookup_index(at)
            .map(|Lookup { holey_index, .. }| match &self.0[holey_index] {
                Element::Value(v) => MaybeHoley::Some(v),
                Element::Hole { .. } => MaybeHoley::Hole,
            })
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn resize(&mut self, to: usize) {
        if to == 0 {
            self.clear();
            return;
        }
        let len = self.compute_len();

        if to <= len {
            let Lookup {
                chunk_start,
                holey_index,
                chunk_count,
            } = self.lookup_index(to - 1).unwrap();
            self.0.truncate(holey_index + 1);
            if let Some(Element::Hole { count }) = self.0.last_mut() {
                let sub = (chunk_start + chunk_count) - to;
                *count -= sub;

                if *count == 0 {
                    self.0.pop();
                }
            }
            // TODO: merge potential holeys? and remove ones at the end
        } else if let Some(Element::Hole { count }) = self.0.last_mut() {
            // make the last hole larger if there is already one
            *count += to - len;
        } else {
            self.0.push(Element::Hole { count: to - len });
        }
    }
    pub fn set(&mut self, at: usize, value: T) {
        if let Some(Lookup {
            chunk_start,
            chunk_count,
            holey_index,
        }) = self.lookup_index(at)
        {
            match &mut self.0[holey_index] {
                Element::Value(existing) => *existing = value,
                Element::Hole { count } if *count == 1 => {
                    self.0[holey_index] = Element::Value(value);
                }
                Element::Hole { .. } => {
                    let left = at - chunk_start;
                    let right = ((chunk_start + chunk_count) - at) - 1;

                    match (left, right) {
                        (0, 0) => {
                            // Setting at 0 @ Hole(1) -> val
                            // Covered by the match arm guard `if *count == 1` above
                            panic!("zero-sized hole after set which should be covered by other arm");
                        }
                        (0, right) => {
                            // Setting at 0 @ [Hole(2)] -> [val, Hole(1)]
                            self.0.splice(
                                holey_index..holey_index + 1,
                                [Element::Value(value), Element::Hole { count: right }],
                            );
                        }
                        (left, 0) => {
                            // Setting at 1 @ [Holey(2)] -> [Hole(1), val]
                            self.0.splice(
                                holey_index..holey_index + 1,
                                [Element::Hole { count: left }, Element::Value(value)],
                            );
                        }
                        (left, right) => {
                            // Setting at 1 @ [Hole(3)] -> [Hole(1), val, Hole(1)]
                            self.0.splice(
                                holey_index..holey_index + 1,
                                [
                                    Element::Hole { count: left },
                                    Element::Value(value),
                                    Element::Hole { count: right },
                                ],
                            );
                        }
                    }
                }
            }
        } else {
            // out of bounds: can just push
            let dist = at - self.compute_len();
            if dist > 0 {
                self.0.push(Element::Hole { count: dist });
            }
            self.0.push(Element::Value(value));
        }
    }

    pub fn push(&mut self, value: T) {
        self.0.push(Element::Value(value));
    }

    pub fn remove(&mut self, at: usize) {
        if let Some(Lookup { holey_index, .. }) = self.lookup_index(at) {
            match &mut self.0[holey_index] {
                Element::Value(_) => drop(self.0.remove(holey_index)),
                Element::Hole { count } => {
                    *count -= 1;
                    if *count == 0 {
                        self.0.remove(holey_index);
                    }
                }
            }
            // TODO: this needs to merge or remove duplicate holes now, and remove ones at the end
        }
    }
}
