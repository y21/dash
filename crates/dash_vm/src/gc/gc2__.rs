use std::mem;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::addr_of;

use super::handle::ObjectVTable;

#[derive(Copy, Clone, Debug)]
enum BinKind {
    B8,
    B32,
    B64,
    Variable,
}

#[derive(Copy, Clone, Debug)]
pub struct ObjectId {
    id: u32,
    // generation: 14 bits
    // bin: 2 bits
    generation_and_bin: u16,
}
const GENERATION_MASK: u16 = 0b0011111111111111;
const BIN_MASK: u16 = 0b1100000000000000;
impl ObjectId {
    pub fn generation_and_bin(&self) -> (u16, u16) {
        (
            self.generation_and_bin & GENERATION_MASK,
            (self.generation_and_bin & BIN_MASK) >> 14,
        )
    }
    pub fn generation(&self) -> u16 {
        self.generation_and_bin().0
    }
    pub fn bin(&self) -> u16 {
        self.generation_and_bin().1
    }
}

struct Relocation {}

struct FreeEntry {
    index: u32,
}

// align S to 4
struct AlignedBuf<const N: usize>
where
    Self: Alignment,
{
    buf: [MaybeUninit<u8>; N],
    _align: <AlignedBuf<N> as Alignment>::Align,
}
impl<const N: usize> AlignedBuf<N>
where
    Self: Alignment,
{
    pub fn new(buf: [MaybeUninit<u8>; N]) -> Self {
        Self {
            buf,
            _align: Default::default(),
        }
    }
}
trait Alignment {
    type Align: Default;
}
#[repr(align(8))]
#[derive(Default)]
struct Align8;
#[repr(align(16))]
#[derive(Default)]
struct Align16;
#[repr(align(32))]
#[derive(Default)]
struct Align32;
#[repr(align(64))]
#[derive(Default)]
struct Align64;
#[repr(align(128))]
#[derive(Default)]
struct Align128;
impl Alignment for AlignedBuf<8> {
    type Align = Align8;
}
impl Alignment for AlignedBuf<16> {
    type Align = Align16;
}
impl Alignment for AlignedBuf<32> {
    type Align = Align32;
}
impl Alignment for AlignedBuf<64> {
    type Align = Align64;
}
impl Alignment for AlignedBuf<128> {
    type Align = Align128; // TODO: not correct
}

pub struct Bin<const N: usize, M>
where
    AlignedBuf<N>: Alignment,
{
    // TODO: make sure to align the entries to T
    storage: Vec<AlignedBuf<N>>,
    free: Vec<FreeEntry>,
    relocations: Vec<Relocation>,
    info: Vec<AllocInfo<M>>,
    generation: u16,
}

struct AllocInfo<M> {
    // usually a vtable
    metadata: M,
    generation: u16,
}

fn n_bin_kind<const N: usize>() -> BinKind {
    const {
        match N {
            8 => BinKind::B8,
            32 => BinKind::B32,
            64 => BinKind::B64,
            _ => panic!("invalid bin size"),
        }
    }
}
const BIN8: u16 = 0b0000000000000000;
const BIN32: u16 = 0b1000000000000000;
const BIN64: u16 = 0b0100000000000000;
const BINVAR: u16 = 0b1100000000000000;
fn n_bin_kind_bits<const N: usize>() -> u16 {
    const {
        match N {
            8 => BIN8,
            32 => BIN32,
            64 => BIN64,
            _ => panic!("invalid bin size"),
        }
    }
}

impl<const N: usize, M> Bin<N, M>
where
    AlignedBuf<N>: Alignment,
{
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            free: Vec::new(),
            relocations: Vec::new(),
            info: Vec::new(),
            generation: 0,
        }
    }
    pub fn alloc<T>(&mut self, value: T, metadata: M) -> ObjectId {
        const {
            assert!(size_of::<T>() == N && N != 0);
        }

        let value = ManuallyDrop::new(value);
        // SAFETY: size_of<T> == N is checked with an inline const, so Dst is not larger than Src
        let value = AlignedBuf::new(unsafe { mem::transmute_copy::<T, [MaybeUninit<u8>; N]>(&value) });

        let free_entry = if const { align_of::<T>() <= N } {
            self.free.pop()
        } else {
            let index = self.free.iter().find_map(|entry| {
                // SAFETY: index is in bounds
                let ptr = unsafe { self.storage.as_ptr().add(entry.index as usize) };
                (ptr as usize % align_of::<T>() == 0).then_some(entry.index)
            });
            index.map(|index| self.free.swap_remove(index as usize))
        };
        let generation_and_bin = self.generation | n_bin_kind_bits::<N>();

        if let Some(free_entry) = free_entry {
            self.info[free_entry.index as usize] = AllocInfo {
                metadata,
                generation: self.generation,
            };
            self.storage[free_entry.index as usize] = value;

            return ObjectId {
                id: free_entry.index,
                generation_and_bin,
            };
        }

        let index: u32 = self.storage.len().try_into().expect("overflow");

        self.storage.push(value);
        self.info.push(AllocInfo {
            metadata,
            generation: self.generation,
        });

        ObjectId {
            id: index,
            generation_and_bin,
        }
    }

    pub fn dealloc(&mut self, id: ObjectId) {
        assert_eq!(self.info[id.id as usize].generation, id.generation());

        self.free.push(FreeEntry { index: id.id });
    }

    pub fn res(&self, id: ObjectId) -> (*const (), *const M) {
        let generation = id.generation();
        assert_eq!(generation, self.generation); // For now. is that even correct?

        let info = unsafe { self.info.as_ptr().add(id.id as usize) };

        assert_eq!(unsafe { (*info).generation }, generation);

        (unsafe { self.storage.as_ptr().add(id.id as usize).cast() }, unsafe {
            addr_of!((*info).metadata)
        })
    }
}

/// `M` is additional metadata to attach to allocations (eg a vtable). Can be `()`
pub struct Allocator<M> {
    bin8: Bin<8, M>,
    bin32: Bin<32, M>,
    bin64: Bin<64, M>,
    binvar: Bin<{ std::mem::size_of::<*const ()>() }, M>,
}

impl<M> Allocator<M> {
    pub fn new() -> Self {
        Self {
            bin8: Bin::new(),
            bin32: Bin::new(),
            bin64: Bin::new(),
            binvar: Bin::new(),
        }
    }
    pub fn alloc<T>(&mut self, value: T, metadata: M) -> ObjectId {
        match mem::size_of::<T>() {
            0 => todo!(),
            1..=8 => self.bin8.alloc(value, metadata),
            9..=32 => self.bin32.alloc(value, metadata),
            33..=64 => self.bin64.alloc(value, metadata),
            _ => {
                let id = self.binvar.alloc(Box::into_raw(Box::new(value)), metadata);
                ObjectId {
                    id: 0,
                    generation_and_bin: id.generation() | BINVAR,
                }
            }
        }
    }
    pub fn dealloc(&mut self, id: ObjectId) {
        match id.generation_and_bin & BIN_MASK {
            BIN8 => self.bin8.dealloc(id),
            BIN32 => self.bin32.dealloc(id),
            BIN64 => self.bin64.dealloc(id),
            BINVAR => {
                self.binvar.dealloc(ObjectId {
                    id: id.id,
                    generation_and_bin: id.generation() | n_bin_kind_bits::<{ size_of::<*const ()>() }>(),
                });
            }
            _ => todo!(),
        }
    }
    pub fn res(&self, id: ObjectId) -> (*const (), *const M) {
        match id.generation_and_bin & BIN_MASK {
            BIN8 => self.bin8.res(id),
            BIN32 => self.bin32.res(id),
            BIN64 => self.bin64.res(id),
            _ => {
                let (res, metadata) = self.binvar.res(ObjectId {
                    id: id.id,
                    generation_and_bin: id.generation() | n_bin_kind_bits::<{ size_of::<*const ()>() }>(),
                });

                (unsafe { *res.cast::<*mut ()>() }, metadata)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Allocator;

    #[test]
    fn size8() {
        let mut gc = Allocator::new();
        let a = dbg!(gc.alloc(42_u64, ()));
        // gc.dealloc(a);
        let b = dbg!(gc.alloc((6_u32, 123213_u32), ()));
        dbg!(unsafe { *gc.res(a).0.cast::<u64>() });
        dbg!(unsafe { *gc.res(b).0.cast::<(u32, u32)>() });
        let c = dbg!(gc.alloc([49u64; 256], ()));
        dbg!(unsafe { (*gc.res(c).0.cast::<[u64; 256]>())[255] });
        gc.dealloc(c);
    }
}
