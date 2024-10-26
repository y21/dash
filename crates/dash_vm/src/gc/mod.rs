use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::any::Any;
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;

use trace::TraceCtxt;

use crate::localscope::LocalScope;
use crate::value::object::{PropertyKey, PropertyValue};
use crate::value::primitive::InternalSlots;
use crate::value::{Typeof, Unrooted, Value};
use crate::Vm;

pub mod persistent;
pub mod trace;

pub type ObjectId = AllocId<&'static ObjectVTable>;

#[repr(C)]
#[allow(clippy::type_complexity)]
pub struct ObjectVTable {
    pub(crate) drop_boxed_gcnode: unsafe fn(*mut ()),
    pub(crate) trace: unsafe fn(*const (), &mut TraceCtxt<'_>),
    pub(crate) debug_fmt: unsafe fn(*const (), &mut core::fmt::Formatter<'_>) -> core::fmt::Result,
    pub(crate) js_get_own_property:
        unsafe fn(*const (), &mut LocalScope<'_>, Value, PropertyKey) -> Result<Unrooted, Unrooted>,
    pub(crate) js_get_own_property_descriptor:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Option<PropertyValue>, Unrooted>,
    pub(crate) js_get_property: unsafe fn(*const (), &mut LocalScope, Value, PropertyKey) -> Result<Unrooted, Unrooted>,
    pub(crate) js_get_property_descriptor:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Option<PropertyValue>, Unrooted>,
    pub(crate) js_set_property:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey, PropertyValue) -> Result<(), Value>,
    pub(crate) js_delete_property: unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Unrooted, Value>,
    pub(crate) js_set_prototype: unsafe fn(*const (), &mut LocalScope<'_>, Value) -> Result<(), Value>,
    pub(crate) js_get_prototype: unsafe fn(*const (), &mut LocalScope<'_>) -> Result<Value, Value>,
    pub(crate) js_apply:
        unsafe fn(*const (), &mut LocalScope<'_>, ObjectId, Value, Vec<Value>) -> Result<Unrooted, Unrooted>,
    pub(crate) js_construct:
        unsafe fn(*const (), &mut LocalScope<'_>, ObjectId, Value, Vec<Value>) -> Result<Unrooted, Unrooted>,
    pub(crate) js_as_any: unsafe fn(*const (), &Vm) -> *const dyn Any,
    pub(crate) js_internal_slots: unsafe fn(*const (), &Vm) -> Option<*const dyn InternalSlots>,
    pub(crate) js_own_keys: unsafe fn(*const (), sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value>,
    pub(crate) js_type_of: unsafe fn(*const (), _: &Vm) -> Typeof,
}

const CHUNK_SIZE: usize = 1 << 12;

macro_rules! object_vtable_for_ty {
    ($ty:ty) => {
        const {
            use $crate::value::object::Object;

            &$crate::gc::ObjectVTable {
                drop_boxed_gcnode: |_ptr| {
                    todo!();
                },
                trace: |ptr, ctxt| unsafe { <$ty as $crate::gc::trace::Trace>::trace(&*(ptr.cast::<$ty>()), ctxt) },
                debug_fmt: |ptr, f| unsafe { <$ty as std::fmt::Debug>::fmt(&*(ptr.cast::<$ty>()), f) },
                js_get_own_property: |ptr, scope, this, key| unsafe {
                    <$ty as Object>::get_own_property(&*(ptr.cast::<$ty>()), scope, this, key)
                },
                js_get_own_property_descriptor: |ptr, scope, key| unsafe {
                    <$ty as Object>::get_own_property_descriptor(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_get_property: |ptr, scope, this, key| unsafe {
                    <$ty as Object>::get_property(&*(ptr.cast::<$ty>()), scope, this, key)
                },
                js_get_property_descriptor: |ptr, scope, key| unsafe {
                    <$ty as Object>::get_property_descriptor(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_set_property: |ptr, scope, key, value| unsafe {
                    <$ty as Object>::set_property(&*(ptr.cast::<$ty>()), scope, key, value)
                },
                js_delete_property: |ptr, scope, key| unsafe {
                    <$ty as Object>::delete_property(&*(ptr.cast::<$ty>()), scope, key)
                },
                js_set_prototype: |ptr, scope, proto| unsafe {
                    <$ty as Object>::set_prototype(&*(ptr.cast::<$ty>()), scope, proto)
                },
                js_get_prototype: |ptr, scope| unsafe { <$ty as Object>::get_prototype(&*(ptr.cast::<$ty>()), scope) },
                js_apply: |ptr, scope, callee, this, args| unsafe {
                    <$ty as Object>::apply(&*(ptr.cast::<$ty>()), scope, callee, this, args)
                },
                js_construct: |ptr, scope, callee, this, args| unsafe {
                    <$ty as Object>::construct(&*(ptr.cast::<$ty>()), scope, callee, this, args)
                },
                js_as_any: |ptr, vm| unsafe { <$ty as Object>::as_any(&*(ptr.cast::<$ty>()), vm) },
                js_internal_slots: |ptr, vm| unsafe {
                    <$ty as Object>::internal_slots(&*(ptr.cast::<$ty>()), vm)
                        .map(|v| v as *const dyn $crate::value::primitive::InternalSlots)
                },
                js_own_keys: |ptr, scope| unsafe { <$ty as Object>::own_keys(&*(ptr.cast::<$ty>()), scope) },
                js_type_of: |ptr, vm| unsafe { <$ty as Object>::type_of(&*(ptr.cast::<$ty>()), vm) },
            }
        }
    };
}

#[derive(Copy, Clone)]
struct ChunkId(u32);
#[derive(Copy, Clone)]
struct LocalAllocId(u16);

/// First 12 bits: local allocation index (same as CHUNK_SIZE)
/// Last 20 bits: chunk id
type PackedInnerAllocId = u32;

pub struct AllocId<M> {
    id_and_chunk: PackedInnerAllocId,
    _metadata: PhantomData<*const M>,
}

impl<M> AllocId<M> {
    pub(crate) fn from_raw(id_and_chunk: PackedInnerAllocId) -> Self {
        Self {
            id_and_chunk,
            _metadata: PhantomData,
        }
    }

    fn from_raw_parts(LocalAllocId(local): LocalAllocId, ChunkId(chunk): ChunkId) -> Self {
        assert!(local < (1 << 12));
        assert!(chunk < (1 << 20));

        Self {
            id_and_chunk: local as u32 | (chunk << 12),
            _metadata: PhantomData,
        }
    }

    fn local(self) -> LocalAllocId {
        const LOCAL_MASK: u32 = (1 << 12) - 1;
        LocalAllocId((self.id_and_chunk & LOCAL_MASK) as u16)
    }

    fn chunk(self) -> ChunkId {
        ChunkId(self.id_and_chunk >> 12)
    }

    pub(crate) fn raw(self) -> PackedInnerAllocId {
        self.id_and_chunk
    }
}
impl<M> fmt::Debug for AllocId<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectId")
            .field("local_id", &self.local().0)
            .field("chunk_id", &self.chunk().0)
            .finish()
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
        let Self {
            id_and_chunk,
            _metadata: _,
        } = *self;
        id_and_chunk == other.id_and_chunk
    }
}
impl<M> Eq for AllocId<M> {}
impl<M> std::hash::Hash for AllocId<M> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self {
            id_and_chunk,
            _metadata: _,
        } = *self;
        id_and_chunk.hash(state);
    }
}

#[derive(Debug)]
struct OutOfSpace;

struct AllocSizeInfo {
    /// The number of padding bytes (excluding the size byte)
    header_padding: u8,
    data_padding: u8,
}

// The general encoding for every allocation is as follows:
// <1 byte padding size, part of padding><padding for header - 1><header><padding for data><1 byte for how far back the metadata in the header is located><data>
struct Chunk {
    data: NonNull<u8>,
    at: usize,
}

impl Chunk {
    fn layout() -> Layout {
        Layout::array::<u8>(CHUNK_SIZE).unwrap()
    }

    pub fn new() -> Self {
        Self {
            data: NonNull::new(unsafe { alloc(Self::layout()).cast() })
                .unwrap_or_else(|| handle_alloc_error(Self::layout())),
            at: 0,
        }
    }

    fn ensure_space(&self, bytes: usize) -> Result<(), OutOfSpace> {
        if self.at + bytes > CHUNK_SIZE {
            Err(OutOfSpace)
        } else {
            Ok(())
        }
    }

    fn push(&mut self, byte: u8) -> Result<(), OutOfSpace> {
        self.ensure_space(1)?;

        unsafe { self.data.add(self.at).write(byte) };
        self.at += 1;
        Ok(())
    }

    fn push_n(&mut self, byte: u8, n: usize) -> Result<(), OutOfSpace> {
        self.ensure_space(n)?;

        unsafe { self.data.add(self.at).write_bytes(byte, n) };
        self.at += n;
        Ok(())
    }

    /// Copies the raw bytes of `T` into the buffer. The `T` must be aligned.
    fn write<T>(&mut self, value: T) -> Result<(), OutOfSpace> {
        let size = size_of::<T>();
        self.ensure_space(size)?;

        // SAFETY: `self.at` is in bounds
        let ptr = unsafe { self.data.add(self.at).cast::<T>() };

        assert!(ptr.is_aligned());
        // SAFETY: `ptr` is properly aligned and there is enough space for a `T`
        unsafe { ptr.write(value) };

        self.at += size;
        Ok(())
    }

    /// Computes the size that an allocation would take up. Returns an error if the allocation won't fit.
    fn size_for_alloc<T, M>(&self) -> Result<AllocSizeInfo, OutOfSpace> {
        let mut total: usize = 1; // The initial padding size

        self.ensure_space(2)?; // Bounds check for the header_padding `add` below
        let header_padding: u8 = unsafe {
            self.data
                .add(self.at + 1)
                .align_offset(align_of::<AllocHeader<M>>())
                .try_into()
                .map_err(|_| OutOfSpace)?
        };

        total += usize::from(header_padding);
        total += size_of::<AllocHeader<M>>();
        total += 1; // 1 Byte back offset to metadata

        self.ensure_space(total + 1)?; // Bounds check for the header_padding `add` below
        let data_padding: u8 = unsafe {
            self.data
                .add(self.at + total)
                .align_offset(align_of::<T>())
                .try_into()
                .map_err(|_| OutOfSpace)?
        };

        total += usize::from(data_padding);
        total += size_of::<T>();

        self.ensure_space(total)?;

        Ok(AllocSizeInfo {
            header_padding,
            data_padding,
        })
    }

    /// Tries to allocate a value in this chunk if there is enough space.
    pub fn try_alloc<T, M>(&mut self, value: T, metadata: M) -> Result<LocalAllocId, (T, M)> {
        let AllocSizeInfo {
            header_padding,
            data_padding,
        } = match self.size_for_alloc::<T, M>() {
            Ok(v) => v,
            Err(OutOfSpace) => return Err((value, metadata)),
        };

        // NB: we've checked that we have enough space in the chunk, so these unwraps ensure that it is correct.

        self.push(header_padding).unwrap();
        self.push_n(0, header_padding.into()).unwrap();

        let metadata_pos = self.at;
        self.write(AllocHeader {
            metadata,
            visited: Cell::new(false),
        })
        .unwrap();

        self.push_n(0, data_padding.into()).unwrap();
        let back_offset_to_metadata = u8::try_from(self.at + 1 - metadata_pos).unwrap();
        self.push(back_offset_to_metadata).unwrap();

        let id = self.at;
        self.write(value).unwrap();

        Ok(LocalAllocId(id.try_into().expect("id < CHUNK_SIZE < usize::MAX")))
    }

    /// # Safety
    /// The given `LocalAllocId` must have been allocated in this chunk.
    pub unsafe fn data(&self, LocalAllocId(id): LocalAllocId) -> *const () {
        // SAFETY: Caller checks that LocalAllocId belongs to this chunk
        // The ids are always indices into the buf
        self.data.add(id as usize).as_ptr().cast()
    }

    pub unsafe fn header<M>(&self, LocalAllocId(id): LocalAllocId) -> *const AllocHeader<M> {
        let ptr = self.data.add(id as usize).as_ptr();
        let offset = *ptr.sub(1);
        // The offset points to the metadata and the metadata is always also the start of the header
        ptr.sub(offset as usize).cast()
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data.as_ptr(), Self::layout());
        }
    }
}

#[repr(C)]
pub struct AllocHeader<M> {
    // Metadata must be at the start
    pub metadata: M,
    pub visited: Cell<bool>,
}

#[derive(Default)]
pub struct Allocator {
    chunks: Vec<Chunk>,
}

impl Allocator {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    fn mk_chunk(&mut self) -> (ChunkId, &mut Chunk) {
        assert!(self.chunks.len() < (1 << 20) - 1, "too many chunks");

        let chunk_id = ChunkId(self.chunks.len() as u32);
        self.chunks.push(Chunk::new());
        (chunk_id, self.chunks.last_mut().unwrap())
    }

    fn last_chunk(&mut self) -> Option<(ChunkId, &mut Chunk)> {
        let len = self.chunks.len();
        let chunk = self.chunks.last_mut()?;
        Some((ChunkId((len - 1) as u32), chunk))
    }

    fn chunk(&self, ChunkId(id): ChunkId) -> &Chunk {
        &self.chunks[id as usize]
    }

    pub fn alloc<T, M>(&mut self, mut value: T, mut metadata: M) -> AllocId<M> {
        const {
            assert!(size_of::<T>() < 1024);
            assert!(size_of::<AllocHeader<M>>() < 1024);
        }

        // TODO: check free list

        if let Some((chunk_id, chunk)) = self.last_chunk() {
            match chunk.try_alloc(value, metadata) {
                Ok(local_id) => return AllocId::from_raw_parts(local_id, chunk_id),
                Err((t, m)) => {
                    value = t;
                    metadata = m;
                    // TODO: since we're going to make a new chunk, add the remaining space to the free list already?
                }
            }
        }

        let (chunk_id, chunk) = self.mk_chunk();
        let local_id = chunk
            .try_alloc(value, metadata)
            .unwrap_or_else(|_| panic!("failed to allocate memory in fully empty chunk"));

        AllocId::from_raw_parts(local_id, chunk_id)
    }

    pub fn alloc_object<O: crate::value::object::Object + 'static>(&mut self, o: O) -> ObjectId {
        self.alloc(o, object_vtable_for_ty!(O))
    }

    pub fn resolve_raw<M>(&self, id: AllocId<M>) -> (*const (), *const AllocHeader<M>) {
        let chunk = self.chunk(id.chunk());
        let local = id.local();
        // SAFETY: the local_id and chunk_id come from the same AllocId
        unsafe { (chunk.data(local), chunk.header(local)) }
    }

    pub fn data<M>(&self, id: AllocId<M>) -> *const () {
        // SAFETY: the local_id and chunk_id come from the same AllocId
        unsafe { self.chunk(id.chunk()).data(id.local()) }
    }

    pub fn header<M>(&self, id: AllocId<M>) -> *const AllocHeader<M> {
        // SAFETY: the local_id and chunk_id come from the same AllocId
        unsafe { self.chunk(id.chunk()).header(id.local()) }
    }

    pub fn rss(&self) -> usize {
        if self.chunks.is_empty() {
            0
        } else {
            (self.chunks.len() - 1) * CHUNK_SIZE + self.chunks.last().unwrap().at
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_simple() {
        let mut allocator = Allocator::new();

        for i in 0..10 {
            let a = 42i16.wrapping_mul(i);
            let b = 328254356i64.wrapping_mul(i as i64);
            let c = i32::MAX;

            let alloc1 = allocator.alloc(a, "asdsada");
            let alloc2 = allocator.alloc(b, "bbbbb");
            let alloc3 = allocator.alloc(c, "cc");

            unsafe {
                assert_eq!(*allocator.data(alloc1).cast::<i16>(), a);
                assert_eq!((*allocator.header(alloc1)).metadata, "asdsada");
                assert_eq!(*allocator.data(alloc2).cast::<i64>(), b);
                assert_eq!((*allocator.header(alloc2)).metadata, "bbbbb");
                assert_eq!(*allocator.data(alloc3).cast::<i32>(), c);
                assert_eq!((*allocator.header(alloc3)).metadata, "cc");
            }
        }
    }

    // #[test]
    // fn gc_works() {
    //     unsafe {
    //         let mut alloc = Allocator::new();

    //         let h1 = alloc.alloc(123.0, ());

    //         assert!(alloc.chunks.len() == 1);

    //         let h2 = register_gc!(f64, gc, 123.4);

    //         assert!(alloc.head == NonNull::new(h1.as_ptr()));
    //         assert!(alloc.tail == NonNull::new(h2.as_ptr()));
    //         assert!(h1.next() == NonNull::new(h2.as_ptr()));
    //         assert!(!h2.flags().contains(HandleFlagsInner::MARKED_VISITED));
    //         assert!(alloc.node_count == 2);

    //         (*h1.as_erased_ptr()).flags.mark();
    //         (*h2.as_erased_ptr()).flags.mark();

    //         assert!((*h1.as_erased_ptr()).flags.is_marked());
    //         assert!((*h2.as_erased_ptr()).flags.is_marked());

    //         alloc.sweep();

    //         // nothing should have changed after GC sweep since all nodes were marked
    //         // they should be unmarked now though
    //         assert!(alloc.head == NonNull::new(h1.as_ptr()));
    //         assert!(alloc.tail == NonNull::new(h2.as_ptr()));
    //         assert!((*h1.as_erased_ptr()).next == NonNull::new(h2.as_ptr()));
    //         assert!(!(*h1.as_erased_ptr()).flags.is_marked());
    //         assert!(!(*h2.as_erased_ptr()).flags.is_marked());
    //         assert!(alloc.node_count == 2);

    //         // add a third node now
    //         let h3 = register_gc!(bool, gc, true);

    //         assert!(alloc.head == NonNull::new(h1.as_ptr()));
    //         assert!(alloc.tail == NonNull::new(h3.as_ptr()));
    //         assert!((*h1.as_erased_ptr()).next == NonNull::new(h2.as_ptr()));
    //         assert!((*h2.as_erased_ptr()).next == NonNull::new(h3.as_ptr()));
    //         assert!(!(*h3.as_erased_ptr()).flags.is_marked());
    //         assert!(alloc.node_count == 3);

    //         // ---

    //         // only mark second node
    //         (*h2.as_erased_ptr()).flags.mark();

    //         alloc.sweep();

    //         // only one node is left: h2
    //         assert!(alloc.node_count == 1);
    //         assert!(alloc.head == NonNull::new(h2.as_ptr()));
    //         assert!(alloc.tail == NonNull::new(h2.as_ptr()));

    //         // final sweep
    //         alloc.sweep();

    //         // nothing left.
    //         assert!(alloc.node_count == 0);
    //         assert!(alloc.head.is_none());
    //         assert!(alloc.tail.is_none());

    //         // test that ExternalValue::replace works
    //         {
    //             // todo!();
    //             // let h4i: Handle = register_gc!(Value, gc, Value::Number(Number(123.4)));
    //             // let ext = ExternalValue::new(h4i);
    //             // assert_eq!(ext.inner(), &Value::Number(Number(123.4)));
    //             // ExternalValue::replace(&ext, Value::Boolean(true));
    //             // assert_eq!(ext.inner(), &Value::Boolean(true));
    //         }

    //         // lastly, test if Gc::drop works correctly. run under miri to see possible leaks
    //         // register_gc!(bool, gc, false);
    //     }
    // }
}
