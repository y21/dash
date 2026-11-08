use std::mem::offset_of;

use crate::Vm;

#[derive(Debug)]
pub struct VecOffsets {
    pub ptr: usize,
    pub len: usize,
    #[cfg_attr(not(test), expect(unused))]
    pub cap: usize,
}

/// Probes the internal memory layout of a `Vec<T>` to determine the offsets of its pointer, length, and capacity fields.
///
/// WARNING: This is very hacky and I'm sorry if you're going to read it.
/// Fundamentally, we want the JIT to be able to push/pop the vm stack, which is represented as a `Vec<Value>`.
/// For that, we need to directly access the internal fields of the `Vec` (pointer & length),
/// however, those fields have an unspecified order and _will_ change randomly, so they can't be hardcoded.
/// Thankfully, even though the layout *is* unspecified, it still does document a number of properties
/// that we can use to probe for the offsets, such as the fact that `Vec<T>` will always be a triplet of (ptr, len, cap).
/// Finally, we run a test under miri to ensure we don't rely on UB when reading from these offsets.
pub fn get_vec_offsets<T>(_: &Vec<T>) -> VecOffsets {
    let vec: Vec<T> = Vec::<T>::with_capacity(4);

    // assumptions don't hold for ZST vecs
    assert_ne!(size_of::<T>(), 0);
    assert_eq!(size_of::<Vec<T>>(), size_of::<usize>() * 3);
    assert!(vec.len() != vec.capacity() && vec.len() != vec.as_ptr().addr() && vec.capacity() != vec.as_ptr().addr());

    let mut ptr = None;
    let mut len = None;
    let mut cap = None;

    let vec_ptr: *const usize = (&raw const vec).cast();

    for i in 0..3 {
        let field = unsafe { vec_ptr.add(i) };
        let value: usize = unsafe { *field };
        let offset = field.addr() - vec_ptr.addr();

        if value == vec.as_ptr().addr() {
            ptr = Some(offset);
        } else if value == vec.capacity() {
            cap = Some(offset);
        } else if value == vec.len() {
            len = Some(offset);
        }
    }

    VecOffsets {
        ptr: ptr.unwrap(),
        len: len.unwrap(),
        cap: cap.unwrap(),
    }
}

/// Returns the offset of the `sp` field relative to the `Vm` struct, in bytes.
pub fn get_sp_offset(vm: &Vm) -> usize {
    let sp_field_offset = vm.frames.sp_field_offset();
    let framestack_offset = offset_of!(Vm, frames);

    framestack_offset + sp_field_offset
}

#[cfg(test)]
mod test {
    use std::ptr::addr_of;

    use dash_middle::compiler::constant::ConstantPool;
    use dash_middle::compiler::{CompileResult, DebugSymbols};
    use dash_middle::indexvec::IndexVec;

    use crate::Vm;
    use crate::frame::{Frame, Sp};
    use crate::jit::codegen::offsets::get_vec_offsets;
    use crate::params::VmParams;
    use crate::value::Value;

    #[test]
    fn vec_value_offsets() {
        let mut vm = Vm::new(VmParams::default());
        // Rerun the probing many times to test reallocations etc
        for i in 0..8 {
            vm.stack.push(Value::number(i as f64));
            let stack = &vm.stack;
            let offsets = get_vec_offsets(stack);
            assert_ne!(offsets.ptr, offsets.len);
            assert_ne!(offsets.ptr, offsets.cap);
            assert_ne!(offsets.len, offsets.cap);
            let stack_ptr = stack as *const Vec<_> as *const u8;
            assert_eq!(
                unsafe { *stack_ptr.add(offsets.ptr).cast::<usize>() },
                stack.as_ptr().addr()
            );
            assert_eq!(unsafe { *stack_ptr.add(offsets.len).cast::<usize>() }, stack.len());
            assert_eq!(unsafe { *stack_ptr.add(offsets.cap).cast::<usize>() }, stack.capacity());
        }
    }

    #[test]
    fn sp_offset() {
        let mut vm = Vm::new(VmParams::default());

        vm.frames
            .push(Frame::from_compile_result(CompileResult {
                cp: ConstantPool::default(),
                debug_symbols: DebugSymbols::default(),
                externals: IndexVec::new(),
                instructions: Vec::new(),
                locals: 0,
                source: "".into(),
            }))
            .unwrap();

        let offset = super::get_sp_offset(&vm);
        eprintln!("{offset:x}");
        assert!(offset > 0);

        for sp in (0..8).map(Sp) {
            vm.frames.set_sp(sp);
            let ptr = unsafe { addr_of!(vm).cast::<u8>().add(offset).cast::<Sp>() };
            assert_eq!(unsafe { *ptr }, sp);
        }
    }
}
