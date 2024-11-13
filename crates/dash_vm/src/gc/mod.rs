use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::any::TypeId;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ptr::{self, NonNull};
use std::{fmt, mem};

use bitflags::bitflags;
use smallvec::SmallVec;
use trace::TraceCtxt;

use crate::frame::This;
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
    pub(crate) trace: unsafe fn(*const (), &mut TraceCtxt<'_>),
    pub(crate) debug_fmt: unsafe fn(*const (), &mut core::fmt::Formatter<'_>) -> core::fmt::Result,
    pub(crate) js_get_own_property:
        unsafe fn(*const (), &mut LocalScope<'_>, This, PropertyKey) -> Result<Unrooted, Unrooted>,
    pub(crate) js_get_own_property_descriptor:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Option<PropertyValue>, Unrooted>,
    pub(crate) js_get_property: unsafe fn(*const (), &mut LocalScope, This, PropertyKey) -> Result<Unrooted, Unrooted>,
    pub(crate) js_get_property_descriptor:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Option<PropertyValue>, Unrooted>,
    pub(crate) js_set_property:
        unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey, PropertyValue) -> Result<(), Value>,
    pub(crate) js_delete_property: unsafe fn(*const (), &mut LocalScope<'_>, PropertyKey) -> Result<Unrooted, Value>,
    pub(crate) js_set_prototype: unsafe fn(*const (), &mut LocalScope<'_>, Value) -> Result<(), Value>,
    pub(crate) js_get_prototype: unsafe fn(*const (), &mut LocalScope<'_>) -> Result<Value, Value>,
    pub(crate) js_apply:
        unsafe fn(*const (), &mut LocalScope<'_>, ObjectId, This, Vec<Value>) -> Result<Unrooted, Unrooted>,
    pub(crate) js_construct:
        unsafe fn(*const (), &mut LocalScope<'_>, ObjectId, This, Vec<Value>) -> Result<Unrooted, Unrooted>,
    pub(crate) js_internal_slots: unsafe fn(*const (), &Vm) -> Option<*const dyn InternalSlots>,
    pub(crate) js_extract_type_raw: unsafe fn(*const (), &Vm, TypeId) -> Option<NonNull<()>>,
    pub(crate) js_own_keys: unsafe fn(*const (), sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value>,
    pub(crate) js_type_of: unsafe fn(*const (), _: &Vm) -> Typeof,
}

const CHUNK_SIZE: usize = 1 << 12;

macro_rules! object_vtable_for_ty {
    ($ty:ty) => {
        const {
            use $crate::value::object::Object;

            &$crate::gc::ObjectVTable {
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
                js_internal_slots: |ptr, vm| unsafe {
                    <$ty as Object>::internal_slots(&*(ptr.cast::<$ty>()), vm)
                        .map(|v| v as *const dyn $crate::value::primitive::InternalSlots)
                },
                js_extract_type_raw: |ptr, vm, id| unsafe {
                    <$ty as Object>::extract_type_raw(&*(ptr.cast::<$ty>()), vm, id)
                },
                js_own_keys: |ptr, scope| unsafe { <$ty as Object>::own_keys(&*(ptr.cast::<$ty>()), scope) },
                js_type_of: |ptr, vm| unsafe { <$ty as Object>::type_of(&*(ptr.cast::<$ty>()), vm) },
            }
        }
    };
}

#[derive(Debug, Copy, Clone)]
struct ChunkId(u32);
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

fn const_checks_for_alloc<T, M>() {
    const {
        assert!(size_of::<T>() < 1024);
        assert!(size_of::<M>() < 256);
        // We could store its `drop_in_place` as well to support it but it's not needed ATP.
        assert!(!mem::needs_drop::<M>());
    }
}

#[derive(Debug)]
struct OutOfSpace;

#[derive(Debug)]
struct AllocSizeInfo {
    /// The number of padding bytes (excluding the size byte)
    header_padding: usize,
    data_padding: usize,
    data_index: u16,
    total: usize,
}

// The general encoding for every allocation is as follows:
// <any padding for metadata - 2  the index is part of the padding>
// <2 byte AllocInfo index>
// <metadata>
// <padding for data>
// <1 byte for how far back the metadata start is located>
// <data>
struct Chunk {
    data: NonNull<u8>,
    info: Vec<AllocInfo>,
    at: usize,
}

bitflags! {
    struct AllocFlags: u8 {
        const INITIALIZED = 1;
        const VISITED = 1 << 1;
    }
}

struct AllocInfo {
    flags: Cell<AllocFlags>,
    total_alloc_size: u16,
    alloc_start: u16,
    data_index: u16, // TODO: this can be computed with alloc_start alone
    drop_in_place: unsafe fn(data: *const ()),
}
impl Clone for AllocInfo {
    fn clone(&self) -> Self {
        const { assert!(!mem::needs_drop::<Self>()) };
        // rustc generates horrible code for the derived Clone impl and we can't derive Copy because of Cell.
        // SAFETY: all fields are trivial and AllocInfo has no drop code, so it's ok to just bitwise copy it
        unsafe { ptr::read(self) }
    }
}

impl Chunk {
    fn layout() -> Layout {
        Layout::array::<u8>(CHUNK_SIZE).unwrap()
    }

    pub fn new() -> Self {
        Self {
            data: NonNull::new(unsafe { alloc(Self::layout()).cast() })
                .unwrap_or_else(|| handle_alloc_error(Self::layout())),
            info: Vec::new(),
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

    fn push_u16(&mut self, n: u16) -> Result<(), OutOfSpace> {
        self.ensure_space(2)?;

        unsafe { self.data.add(self.at).cast::<u16>().write_unaligned(n) };
        self.at += 2;
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

    /// Computes the size that an allocation would take up at a given position. Returns an error if the allocation won't fit in the given `size`.
    fn size_for_alloc_at<T, M>(&self, at: usize) -> Result<AllocSizeInfo, OutOfSpace> {
        // Start with the alloc info index + padding.
        let mut total: usize = 2;

        // Make sure that `additional` extra bytes fit.
        let ensure = |total: usize, additional: usize| {
            if at + total + additional > CHUNK_SIZE {
                Err(OutOfSpace)
            } else {
                Ok(())
            }
        };

        ensure(total, 1)?; // Bounds check for the header_padding `add` below
        let header_padding = unsafe { self.data.add(at + total).align_offset(align_of::<M>()) };
        if header_padding > u8::MAX as usize {
            return Err(OutOfSpace);
        }

        total += header_padding;
        total += size_of::<M>();
        total += 1; // 1 Byte back offset to metadata

        ensure(total, 1)?; // Bounds check for the header_padding `add` below
        let data_padding = unsafe { self.data.add(at + total).align_offset(align_of::<T>()) };
        if data_padding > u8::MAX as usize {
            return Err(OutOfSpace);
        }

        total += data_padding;
        let data_index = (at + total) as u16;
        total += size_of::<T>();

        self.ensure_space(total)?;

        Ok(AllocSizeInfo {
            header_padding,
            data_padding,
            data_index,
            total,
        })
    }

    /// Tries to reuse an existing allocation for a `T`. The returned `LocalAllocId` can be different in case of different alignment
    pub unsafe fn try_alloc_at<T, M>(
        &mut self,
        allocation_start: usize,
        old_total_allocation_size: u16,
        info_id: Option<u16>,
        value: T,
        metadata: M,
    ) -> Result<(LocalAllocId, usize), (T, M)> {
        const_checks_for_alloc::<T, M>();

        let AllocSizeInfo {
            header_padding,
            data_padding,
            data_index,
            total: total_alloc_size,
        } = match self.size_for_alloc_at::<T, M>(allocation_start) {
            // Make sure that even if it fits in the chunk, it doesn't overflow the old allocation.
            Ok(v) if v.total > old_total_allocation_size as usize => return Err((value, metadata)),
            Ok(v) => v,
            Err(OutOfSpace) => return Err((value, metadata)),
        };

        let alloc_info = AllocInfo {
            flags: Cell::new(AllocFlags::INITIALIZED),
            total_alloc_size: total_alloc_size as u16,
            alloc_start: allocation_start as u16,
            data_index,
            drop_in_place: unsafe {
                mem::transmute::<unsafe fn(*mut T), unsafe fn(*const ())>(ptr::drop_in_place::<T> as _)
            },
        };
        let info_id = if let Some(info_id) = info_id {
            self.info[info_id as usize] = alloc_info;
            info_id
        } else {
            let info_id: u16 = self.info.len().try_into().unwrap();
            self.info.push(alloc_info);
            info_id
        };

        let old_at = self.at;
        self.at = allocation_start;
        let id = self.write_alloc_data(info_id, header_padding, data_padding, value, metadata);

        assert!(self.at <= allocation_start + total_alloc_size);
        self.at = old_at;

        Ok((id, total_alloc_size))
    }

    /// Tries to allocate a value in this chunk if there is enough space.
    pub fn try_alloc<T, M>(&mut self, value: T, metadata: M) -> Result<(LocalAllocId, usize), (T, M)> {
        const_checks_for_alloc::<T, M>();

        let AllocSizeInfo {
            header_padding,
            data_padding,
            data_index,
            total: total_alloc_size,
        } = match self.size_for_alloc_at::<T, M>(self.at) {
            Ok(v) => v,
            Err(OutOfSpace) => return Err((value, metadata)),
        };

        let info_id: u16 = self.info.len().try_into().unwrap();
        self.info.push(AllocInfo {
            flags: Cell::new(AllocFlags::INITIALIZED),
            total_alloc_size: total_alloc_size as u16,
            data_index,
            alloc_start: self.at as u16,
            drop_in_place: unsafe {
                mem::transmute::<unsafe fn(*mut T), unsafe fn(*const ())>(ptr::drop_in_place::<T> as _)
            },
        });

        let id = self.write_alloc_data(info_id, header_padding, data_padding, value, metadata);
        assert_eq!(id.0, data_index);

        Ok((id, total_alloc_size))
    }

    fn write_alloc_data<T, M>(
        &mut self,
        info_id: u16,
        header_padding: usize,
        data_padding: usize,
        value: T,
        metadata: M,
    ) -> LocalAllocId {
        self.at += header_padding;
        self.push_u16(info_id).unwrap();

        let metadata_pos = self.at;

        self.write(metadata).unwrap();

        self.at += data_padding;
        let back_offset_to_metadata = u8::try_from(self.at + 1 - metadata_pos).unwrap();
        self.push(back_offset_to_metadata).unwrap();

        let id = self.at;
        self.write(value).unwrap();

        LocalAllocId(id.try_into().expect("id < CHUNK_SIZE < usize::MAX"))
    }

    /// # Safety
    /// The given `LocalAllocId` must have been allocated in this chunk.
    pub unsafe fn data(&self, LocalAllocId(id): LocalAllocId) -> *const () {
        // SAFETY: Caller checks that LocalAllocId belongs to this chunk
        // The ids are always indices into the buf
        self.data.add(id as usize).as_ptr().cast()
    }

    pub unsafe fn metadata<M>(&self, LocalAllocId(id): LocalAllocId) -> *const M {
        let ptr = self.data.add(id as usize).as_ptr();
        let offset = *ptr.sub(1);
        ptr.sub(offset as usize).cast()
    }

    pub unsafe fn info_id(&self, LocalAllocId(id): LocalAllocId) -> u16 {
        let ptr = self.data.add(id as usize).as_ptr();
        let metadata_offset = *ptr.sub(1);
        let info_index_offset = metadata_offset as usize + size_of::<u16>();
        ptr.sub(info_index_offset).cast::<u16>().read_unaligned()
    }

    pub unsafe fn info(&self, id: LocalAllocId) -> &AllocInfo {
        &self.info[self.info_id(id) as usize]
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data.as_ptr(), Self::layout());
        }
    }
}

#[derive(Debug)]
struct FreeListEntry {
    chunk: ChunkId,
    allocation_start_index: u16,
    info_id: Option<u16>,
}

#[derive(Default)]
pub struct Allocator {
    chunks: Vec<Chunk>,
    /// Maps from `total_alloc_size` to a chunk the allocation start index in it
    free_list: BTreeMap<u16, SmallVec<[FreeListEntry; 1]>>,
    rss: usize,
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            free_list: BTreeMap::new(),
            rss: 0,
        }
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

    fn alloc_in_free_list<T, M>(&mut self, mut value: T, mut metadata: M) -> Result<AllocId<M>, (T, M)> {
        let approximated_size = size_of::<T>() as u16 + size_of::<M>() as u16;

        let range = self.free_list.range_mut(approximated_size..);

        for (&exact_size, alloc_ids) in range {
            let FreeListEntry {
                chunk: chunk_id,
                allocation_start_index,
                info_id,
            } = *alloc_ids.last().unwrap();

            let chunk = &mut self.chunks[chunk_id.0 as usize];
            match unsafe { chunk.try_alloc_at(allocation_start_index.into(), exact_size, info_id, value, metadata) } {
                Ok((local_alloc_id, size)) => {
                    self.rss += size;
                    alloc_ids.pop();
                    if alloc_ids.is_empty() {
                        self.free_list.remove(&exact_size);
                    }

                    let remaining_size = exact_size - size as u16;
                    if remaining_size > 12 {
                        // If the difference is large enough that it could fit another small allocation,
                        // push the unused space to avoid wasting space.
                        self.free_list.entry(remaining_size).or_default().push(FreeListEntry {
                            allocation_start_index: allocation_start_index + size as u16,
                            chunk: chunk_id,
                            info_id: None,
                        });
                    }

                    return Ok(AllocId::from_raw_parts(local_alloc_id, chunk_id));
                }
                Err((t, m)) => {
                    value = t;
                    metadata = m;
                }
            }
        }

        Err((value, metadata))
    }

    pub fn alloc<T, M>(&mut self, mut value: T, mut metadata: M) -> AllocId<M> {
        match self.alloc_in_free_list(value, metadata) {
            Ok(id) => return id,
            Err((t, m)) => {
                value = t;
                metadata = m;
            }
        }

        if let Some((chunk_id, chunk)) = self.last_chunk() {
            match chunk.try_alloc(value, metadata) {
                Ok((local_id, alloc_size)) => {
                    self.rss += alloc_size;
                    return AllocId::from_raw_parts(local_id, chunk_id);
                }
                Err((t, m)) => {
                    value = t;
                    metadata = m;

                    let current_chunk_pos = chunk.at;
                    let remaining_chunk_size = (CHUNK_SIZE - current_chunk_pos) as u16;
                    if remaining_chunk_size > 12 {
                        // We're going to make a new chunk, so put the chunk's remaining space in the free list
                        self.free_list
                            .entry(remaining_chunk_size)
                            .or_default()
                            .push(FreeListEntry {
                                allocation_start_index: current_chunk_pos as u16,
                                chunk: chunk_id,
                                info_id: None,
                            });
                    }
                }
            }
        }

        let (chunk_id, chunk) = self.mk_chunk();
        let (local_id, alloc_size) = chunk
            .try_alloc(value, metadata)
            .unwrap_or_else(|_| panic!("failed to allocate memory in fully empty chunk"));
        self.rss += alloc_size;

        AllocId::from_raw_parts(local_id, chunk_id)
    }

    pub fn alloc_object<O: crate::value::object::Object + 'static>(&mut self, o: O) -> ObjectId {
        self.alloc(o, object_vtable_for_ty!(O))
    }

    pub fn resolve_raw<M>(&self, id: AllocId<M>) -> (*const (), *const M) {
        let chunk = self.chunk(id.chunk());
        let local = id.local();
        // SAFETY: the local_id and chunk_id come from the same AllocId
        unsafe { (chunk.data(local), chunk.metadata(local)) }
    }

    fn info<M>(&self, id: AllocId<M>) -> &AllocInfo {
        unsafe { self.chunk(id.chunk()).info(id.local()) }
    }

    pub fn data<M>(&self, id: AllocId<M>) -> *const () {
        // SAFETY: the local_id and chunk_id come from the same AllocId
        unsafe { self.chunk(id.chunk()).data(id.local()) }
    }

    pub fn metadata<M>(&self, id: AllocId<M>) -> *const M {
        // SAFETY: the local_id and chunk_id come from the same AllocId
        unsafe { self.chunk(id.chunk()).metadata(id.local()) }
    }

    /// # Safety
    /// Callers must ensure that objects that are deleted as a result of not having been marked are never accessed again.
    /// In practice this is ensured by marking everything reachable first.
    pub unsafe fn sweep(&mut self) {
        for (chunk_id, chunk) in self.chunks.iter().enumerate() {
            let chunk_id = ChunkId(chunk_id as u32);

            let info_iter = chunk
                .info
                .iter()
                .enumerate()
                .filter(|(_, info)| info.flags.get().contains(AllocFlags::INITIALIZED));

            for (info_id, info) in info_iter {
                if !info.flags.get().contains(AllocFlags::VISITED) {
                    // Object did not get visited
                    let ptr = chunk.data.as_ptr().add(info.data_index as usize).cast::<()>();
                    (info.drop_in_place)(ptr);
                    info.flags.set(info.flags.get() - AllocFlags::INITIALIZED);
                    self.free_list
                        .entry(info.total_alloc_size)
                        .or_default()
                        .push(FreeListEntry {
                            allocation_start_index: info.alloc_start,
                            chunk: chunk_id,
                            info_id: Some(info_id as u16),
                        });

                    self.rss -= info.total_alloc_size as usize;
                } else {
                    info.flags.set(info.flags.get() - AllocFlags::VISITED);
                }
            }
        }
    }

    pub fn rss(&self) -> usize {
        self.rss
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn reclaim() {
        let mut allocator = Allocator::new();

        allocator.alloc([42i32; 8], ());
        allocator.alloc(42i16, ());
        allocator.alloc(42i64, ());
        allocator.alloc(100i128, ());
        allocator.alloc(42i16, ());
        allocator.alloc(42i64, ());
        allocator.alloc(42i16, ());

        unsafe { allocator.sweep() };
        allocator.alloc(46i16, ());

        allocator.alloc(42u8, ());
        allocator.alloc(42u8, ());
        allocator.alloc(42u64, ());
        allocator.alloc(42u32, ());
    }

    #[test]
    fn alloc_simple() {
        let mut allocator = Allocator::new();

        for i in 0..5 {
            let a = 42i16.wrapping_mul(i);
            let b = 328254356i64.wrapping_mul(i as i64);
            let c = i32::MAX;

            let alloc1 = allocator.alloc(a, "asdsada");
            let alloc2 = allocator.alloc(b, "bbbbb");
            let alloc3 = allocator.alloc(c, "cc");
            allocator.alloc(vec![1, 2], "asdasd");

            unsafe {
                assert_eq!(*allocator.data(alloc1).cast::<i16>(), a);
                assert_eq!(*allocator.metadata(alloc1), "asdsada");
                assert_eq!(*allocator.data(alloc2).cast::<i64>(), b);
                assert_eq!(*allocator.metadata(alloc2), "bbbbb");
                assert_eq!(*allocator.data(alloc3).cast::<i32>(), c);
                assert_eq!(*allocator.metadata(alloc3), "cc");
            }
        }

        unsafe { allocator.sweep() };

        let mut vm = Vm::new(Default::default());

        for _ in 0..5 {
            let a = vm.alloc.alloc(i64::MAX, ());
            let b = vm.alloc.alloc(i8::MAX, ());
            let c = vm.alloc.alloc(i32::MAX, ());
            assert_eq!(unsafe { *vm.alloc.data(a).cast::<i64>() }, i64::MAX);
            assert_eq!(unsafe { *vm.alloc.data(b).cast::<i8>() }, i8::MAX);
            assert_eq!(unsafe { *vm.alloc.data(c).cast::<i32>() }, i32::MAX);
            unsafe { vm.alloc.sweep() };
        }
    }
}
