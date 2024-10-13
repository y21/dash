use core::fmt;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use smallvec::SmallVec;

use crate::object_vtable_for_ty;

use super::buf::AlignedBuf;

pub struct Allocator {
    /// The main storage.
    ///
    /// The general encoding for every allocation is as follows:
    /// <1 byte padding size><padding><metadata><padding><1 byte for how far back the metadata is located><data>
    ///
    /// Padding ensures that <metadata> and <data> is properly aligned.
    /// Given the data index, we can compute the metadata index based on the previous byte in the buffer.
    /// The padding size allows walking the buffer.
    buf: AlignedBuf,
    free_list: BTreeMap<usize, SmallVec<[FreeListEntry; 1]>>,
}

struct FreeListEntry {}

#[repr(C)] // metadata must be at the start
pub struct AllocHeader<M> {
    pub metadata: M,
    pub visited: Cell<bool>,
    // Note: this allocation CANNOT be moved if refcount > 0 (?)
    // because `Persistent` stores raw pointers
    pub refcount: Cell<u32>,
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            buf: AlignedBuf::new(),
            free_list: BTreeMap::new(),
        }
    }

    pub fn alloc<T, M>(&mut self, value: T, metadata: M) -> AllocId<M> {
        const {
            // Enforce a max. alignment because the vec may reallocate which could unalign everything again.
            assert!(align_of::<AllocHeader<M>>() <= 8);
            assert!(align_of::<T>() <= 8);
            // AllocHeader must be 1 byte as that's what the byte that precedes the data stores
            assert!(size_of::<AllocHeader<M>>() <= 256);
            // ZSTs are unsupported for now
            assert!(size_of::<AllocHeader<M>>() > 0 && size_of::<T>() > 0);
        }

        // TODO: check if there's an entry in free_list that's >= size_of<T> but < size_of<T> * 2
        // and sizeof header etc. works

        // First, add padding bytes and the size in front of the header.

        self.buf.reserve(2);

        // SAFETY: we reserved two bytes, so .add(len).add(1) is always guaranteed in bounds
        if unsafe { self.buf.insertion_point().add(1).cast::<AllocHeader<M>>().is_aligned() } {
            self.buf.push(0);
        } else {
            let padding: u8 = unsafe {
                (self
                    .buf
                    .insertion_point()
                    .add(1)
                    .align_offset(align_of::<AllocHeader<M>>()))
                .try_into()
                .expect("allocation header requires more than 256 padding bytes")
            };

            self.buf.push(padding);
            self.buf.push_n(0, padding as usize);
        };

        let metadata_idx = self.buf.len();

        // SAFETY: AllocHeader<M> will be properly aligned in the buf due to the padding insertion above
        unsafe {
            self.buf.write(AllocHeader {
                metadata,
                visited: Cell::new(false),
                refcount: Cell::new(0),
            })
        };

        // Next, add padding bytes to align the value itself.
        // Same deal as above, reserve enough extra space so that .add(len).add(1) is in bounds
        self.buf.reserve(2);

        if unsafe { !self.buf.insertion_point().add(1).cast::<T>().is_aligned() } {
            // Data is not aligned. Add padding bytes.
            let padding = unsafe { self.buf.insertion_point().add(1).align_offset(align_of::<T>()) };

            self.buf.push_n(0, padding);
        }
        self.buf.push((self.buf.len() + 1 - metadata_idx).try_into().unwrap());

        let data_idx = self.buf.len();

        // SAFETY: we add padding bytes above, so the `T` is properly aligned
        unsafe { self.buf.write(value) };

        AllocId {
            id: data_idx.try_into().expect("id overflow"),
            _metadata: PhantomData,
        }
    }

    pub fn alloc_object<O: crate::value::object::Object + 'static>(&mut self, o: O) -> super::ObjectId {
        self.alloc(o, object_vtable_for_ty!(O))
    }

    pub fn resolve_raw<M>(&self, id: AllocId<M>) -> (*const (), *const AllocHeader<M>) {
        // TODO: when we have generations and compacting, the index can be OOB
        let ptr = unsafe { self.buf.as_ptr().add(id.id as usize) };
        let offset = unsafe { *ptr.sub(1) };
        let metadata = unsafe { ptr.sub(offset as usize) };
        (ptr.cast(), metadata.cast())
    }

    pub fn data<M>(&self, id: AllocId<M>) -> *const () {
        // TODO: implement resolve_raw in terms of data and header instead of the wrong way around..?
        self.resolve_raw(id).0
    }

    pub fn header<M>(&self, id: AllocId<M>) -> *const AllocHeader<M> {
        // AllocHeader<M> is the first field and repr(C), so `M` can be reinterpreted
        self.resolve_raw(id).1.cast()
    }
}

pub struct AllocId<M> {
    id: u32,
    _metadata: PhantomData<*const M>,
}
impl<M> AllocId<M> {
    pub fn from_raw(id: u32) -> Self {
        Self {
            id,
            _metadata: PhantomData,
        }
    }
    pub fn raw(self) -> u32 {
        self.id
    }
}
impl<M> fmt::Debug for AllocId<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ObjectId").field(&self.id).finish()
    }
}
impl<M> Copy for AllocId<M> {}
impl<M> Clone for AllocId<M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<M> PartialEq for AllocId<M> {
    fn eq(&self, other: &Self) -> bool {
        let Self { id, _metadata: _ } = self;
        *id == other.id
    }
}
impl<M> Eq for AllocId<M> {}
impl<M> std::hash::Hash for AllocId<M> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self { id, _metadata: _ } = self;
        id.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use crate::gc::gc2::AlignedBuf;
    use crate::gc::handle::ObjectVTable;
    use crate::statics::Statics;
    use crate::value::object::Object;
    use crate::value::primitive::Number;
    use crate::value::Unpack;
    use crate::{object_vtable_for_ty, Vm};

    use super::Allocator;

    #[test]
    fn alloc_simple() {
        println!();
        println!();
        println!();

        let mut allocator = Allocator::new();

        // let id1 = allocator.alloc(42_u64, "cool");
        // let id2 = allocator.alloc(49u32, ());
        // let id3 = allocator.alloc(Number(43247.5234f64), object_vtable_for_ty!(Number));
        // // let id2 = allocator.alloc("test", ());

        // println!("{id1:?}: {}", unsafe { *allocator.resolve_raw(id1).0.cast::<u64>() });
        // // <Number as Object>::apply;
        // println!("{id2:?}: {}", unsafe { *(allocator.resolve_raw(id2).0).cast::<u32>() });
        // println!("{id3:?}: {}", unsafe {
        //     *(allocator.resolve_raw(id3).0).cast::<Number>()
        // });

        let mut vm = Vm::new(Default::default());
        dbg!(vm.alloc.buf.len());
        dbg!(vm.eval("1", Default::default()).unwrap().try_prim().unwrap().unpack());
        dbg!(vm.alloc.buf.len());

        // println!("{id2:?}: {}", unsafe { *allocator.resolve_raw(id2).0.cast::<&str>() });

        // for _ in 0..1000 {
        //     let t = std::time::Instant::now();
        // }

        // dbg!(unsafe { *allocator.resolve_raw(a).0.cast::<u64>() });
        // dbg!(allocator.alloc(43_u64, ()));
    }
}
