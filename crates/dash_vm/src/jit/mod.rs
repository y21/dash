use std::ffi::c_void;
use std::mem::{self, offset_of};
use std::ptr::{self, NonNull};

use dash_middle::compiler::instruction::{Instruction, IntrinsicOperation};

use crate::Vm;
use crate::dispatch::{DispatchContext, INSTRUCTION_LUT};
use crate::frame::Ip;
use crate::jit::jumpresolver::InternalLabel;
use crate::localscope::LocalScope;
use crate::value::Unrooted;

mod jumpresolver;
mod x86;

#[derive(Debug)]
pub struct MmapFn {
    ptr: NonNull<c_void>,
    len: usize,
}

impl MmapFn {
    unsafe fn call0<R>(&self) -> R {
        let f = unsafe { mem::transmute::<_, unsafe extern "C" fn() -> R>(self.ptr.as_ptr()) };
        unsafe { f() }
    }
    unsafe fn call1<T, R>(&self, arg: T) -> R {
        let f = unsafe { mem::transmute::<_, unsafe extern "C" fn(T) -> R>(self.ptr.as_ptr()) };
        unsafe { f(arg) }
    }
    unsafe fn call2<T1, T2, R>(&self, arg1: T1, arg2: T2) -> R {
        let f = unsafe { mem::transmute::<_, unsafe extern "C" fn(T1, T2) -> R>(self.ptr.as_ptr()) };
        unsafe { f(arg1, arg2) }
    }
}

impl Drop for MmapFn {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr.as_ptr(), self.len);
        }
    }
}

fn mmap_jit_fn(code: &[u8]) -> MmapFn {
    let ptr = NonNull::new(unsafe {
        libc::mmap(
            ptr::null_mut(),
            code.len(),
            libc::PROT_READ | libc::PROT_EXEC | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    })
    .expect("failed to mmap jit region");

    unsafe { ptr.copy_from_nonoverlapping(NonNull::new(code.as_ptr().cast_mut()).unwrap().cast(), code.len()) };
    MmapFn { ptr, len: code.len() }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub enum HandlerStubStatus {
    Normal,
    Exception,
}

#[repr(C)]
pub union HandlerStubPayload {
    value: Unrooted,
    uninit: (),
}

#[repr(C)]
struct HandlerStubReturn {
    status: HandlerStubStatus,
    payload: HandlerStubPayload,
}

#[repr(C)]
pub struct JitReturn(HandlerStubReturn);

/// Called by JIT code to call a handler.
extern "C" fn handler_stub(vm: &mut Vm, handler: u8, ip: u32) -> HandlerStubReturn {
    vm.frames.set_ip(Ip(ip));
    let cx = DispatchContext::new(vm.scope());

    let result = INSTRUCTION_LUT[handler as usize](cx);

    match result {
        Ok(Some(_)) => todo!(),
        Ok(None) => HandlerStubReturn {
            status: HandlerStubStatus::Normal,
            payload: HandlerStubPayload { uninit: () },
        },
        Err(exception) => HandlerStubReturn {
            status: HandlerStubStatus::Exception,
            payload: HandlerStubPayload { value: exception },
        },
    }
}

#[repr(C)]
struct JitVtable {
    stub_fn: extern "C" fn(&mut Vm, u8, u32) -> HandlerStubReturn,
    last_value_is_truthy: extern "C" fn(&mut Vm, bool) -> bool,
}

extern "C" fn last_value_is_truthy(vm: &mut Vm, pop: bool) -> bool {
    let result = vm.stack.last().unwrap().clone().is_truthy(&mut vm.scope());
    if pop {
        vm.stack.pop();
    }
    result
}

static JIT_VTABLE: JitVtable = JitVtable {
    stub_fn: handler_stub,
    last_value_is_truthy,
};

/*
Parameter registers: RDI, RSI, RDX, RCX, R8, R9

Prologue:
- Vm* is in RDI,
- JitVtable* is in RSI,

- Push RBP (just so the stack is aligned), R12, R13 onto the stack
- Move rsp into rbp (not really necessary for this I think but lets do it anyway, need reg2-reg moves either way)
- Move rdi into r12
- Move the stub handler, i.e. [rsi + 0] into r13
- Make sure rsp is 16-byte aligned (since we push 3 regs, it already is)


Generic stub call:

- Move r12 into rdi (Vm*),
- Move the handler number into rsi
- Call 14 (Return value is then in rax)

(( can ignore this for the first test ))
- Compare rax to 0 (HandlerStubStatus::Normal)
- If not equal, jump to NOT_NORMAL_HANDLER

at the end:
- Move STATUS_NORMAL INTO rax (leave rdx uninit)
- Pop R13, R12, RBP, in that order (as we pushed it)
- Return


*/

pub fn compile_loop_region(scope: &mut LocalScope<'_>, start: Ip, end: Ip) {
    let func = scope.frames.with_current_bytecode(|bytecode| {
        fn target_from_relative(next_bc_ip: u32, rel: i16) -> Ip {
            let target = next_bc_ip as i64 + rel as i64;
            assert!(target >= 0, "computed negative bytecode ip target: {target}");
            Ip(target as u32)
        }

        let bytecode = &bytecode[start.0 as usize..end.0 as usize];

        let mut x86 = x86::Emitter::new(bytecode.len());

        // Prologue
        x86.push(x86::Register::Rbp);
        x86.mov_reg_reg(x86::Register::Rbp, x86::Register::Rsp);
        x86.push(x86::Register::R12);
        x86.mov_reg_reg(x86::Register::R12, x86::Register::Rdi);
        x86.push(x86::Register::R13);

        // Move the stub handler into r13
        x86.mov_reg_mem_u8(
            x86::Register::R13,
            x86::Register::Rsi,
            offset_of!(JitVtable, stub_fn).try_into().unwrap(),
        );
        x86.push(x86::Register::R14);
        x86.mov_reg_reg(x86::Register::R14, x86::Register::Rsi);
        x86.sub_rsp_imm8(8); // align to 16 bytes

        // ... Body ...
        fn emit_stub_for_instr(x86: &mut x86::Emitter, instr: Instruction, ip: u32) {
            x86.mov_reg_reg(x86::Register::Rdi, x86::Register::R12);
            x86.mov_reg_imm32(x86::Register::Rsi, instr as u32);
            x86.mov_reg_imm32(x86::Register::Rdx, ip);
            x86.call_reg(x86::Register::R13);
            x86.test_reg_reg(x86::Register::Eax, x86::Register::Eax);
            x86.jne_internal_label(InternalLabel::StubStatusHandler);
        }

        let mut i = 0;
        while i < bytecode.len() {
            x86.mark_bytecode_ip(Ip(i as u32));

            let instr = Instruction::from_repr(bytecode[i]).unwrap();
            i += 1;

            // IP for the operands *in the full bytecode* of the function (not the sliced loop bytecode).
            let operands_absolute_ip = start.0 + i as u32;

            match instr {
                Instruction::JmpFalseP => {
                    let target_rel = i16::from_le_bytes([bytecode[i], bytecode[i + 1]]);
                    i += 2;
                    let target_bc_ip = target_from_relative(i as u32, target_rel);

                    x86.mov_reg_mem_u8(
                        x86::Register::Rax,
                        x86::Register::R14,
                        offset_of!(JitVtable, last_value_is_truthy).try_into().unwrap(),
                    );
                    x86.mov_reg_reg(x86::Register::Rdi, x86::Register::R12);
                    x86.mov_reg_imm32(x86::Register::Rsi, 1);
                    x86.call_reg(x86::Register::Rax);
                    x86.cmp_reg_al_imm8(1);
                    x86.jne_bytecode_ip(target_bc_ip);
                }
                Instruction::LoopBackJmp => {
                    let target_rel = i16::from_le_bytes([bytecode[i + 1], bytecode[i + 2]]);
                    i += 3;
                    let target_bc_ip = target_from_relative(i as u32, target_rel);

                    x86.jmp_bytecode_ip(target_bc_ip);
                }
                Instruction::IntrinsicOp => {
                    let intrinsic = IntrinsicOperation::from_repr(bytecode[i]).unwrap();
                    i += 1;
                    match intrinsic {
                        IntrinsicOperation::LtNumLConstR | IntrinsicOperation::PostfixIncLocalNum => {
                            i += 1;
                            emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                        }
                        IntrinsicOperation::LtNumLConstR32 => {
                            i += 4;
                            emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                        }
                        _ => todo!(),
                    }
                }
                Instruction::LdLocal => {
                    i += 2;
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Pop => {
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                other => todo!("{other:?} @ {i:x}"),
            }
        }

        // Exit branch (end-of-loop/end-of-bytecode)
        assert!(i == bytecode.len());
        x86.mark_bytecode_ip(Ip(i as u32));
        x86.mov_reg_imm32(x86::Register::Eax, 0);

        // Epilogue
        x86.mark_internal_label(InternalLabel::Epilogue);
        x86.add_rsp_imm8(8);
        x86.pop(x86::Register::R14);
        x86.pop(x86::Register::R13);
        x86.pop(x86::Register::R12);
        x86.pop(x86::Register::Rbp);
        x86.ret();

        x86.mark_internal_label(InternalLabel::StubStatusHandler);
        x86.jmp_internal_label(InternalLabel::Epilogue);

        mmap_jit_fn(x86.buffer())
    });

    let res = unsafe { func.call2::<&mut Vm, &JitVtable, JitReturn>(scope, &JIT_VTABLE) };
    assert!(
        res.0.status == HandlerStubStatus::Normal,
        "JITed code returned non-normal status: {:?}",
        res.0.status
    );
}
