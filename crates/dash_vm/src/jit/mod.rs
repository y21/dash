use std::mem::{MaybeUninit, offset_of};
use std::rc::Rc;

use dash_middle::compiler::instruction::{Instruction, IntrinsicOperation};

use crate::Vm;
use crate::dispatch::{DispatchContext, INSTRUCTION_LUT};
use crate::frame::Ip;
use crate::jit::jumpresolver::InternalLabel;
use crate::jit::mmap::MmapFn;
use crate::localscope::LocalScope;
use crate::value::Unrooted;

mod jumpresolver;
mod mmap;
mod state;
mod x86;

pub use state::State;

#[derive(Debug)]
pub struct JitFnHandle(Rc<MmapFn>);

impl JitFnHandle {
    pub fn call(&self, vm: &mut Vm) -> JitReturn {
        let mut out = MaybeUninit::<JitOutData>::zeroed();

        let ret = self
            .0
            .call3::<&mut Vm, &JitVtable, &mut MaybeUninit<JitOutData>, InternalJitReturn>(vm, &JIT_VTABLE, &mut out);

        // SAFETY: ip is always in bounds
        let ip = unsafe { &raw const (*out.as_ptr()).ip };

        match ret.status {
            // SAFETY: jit initializes out->ip for normal returns
            HandlerStubStatus::Normal => JitReturn::Normal { ip: Ip(unsafe { *ip }) },
            HandlerStubStatus::Exception => {
                let exception = unsafe { ret.payload.value };
                JitReturn::Exception { value: exception }
            }
        }
    }
}

type InternalJitReturn = HandlerStubReturn;

#[derive(Debug)]
pub enum JitReturn {
    /// JIT code returns normally at the given bytecode ip.
    Normal {
        ip: Ip,
    },
    Exception {
        value: Unrooted,
    },
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

/// Called by JIT code to call a handler and synchronize any vm state.
extern "C" fn handler_stub(vm: &mut Vm, _: *mut JitOutData, handler: u8, ip: u32) -> HandlerStubReturn {
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
    stub_fn: extern "C" fn(&mut Vm, *mut JitOutData, u8, u32) -> HandlerStubReturn,
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

#[repr(C)]
struct JitOutData {
    ip: u32,
}

fn compile_uncached(scope: &mut LocalScope<'_>, start: Ip, end: Ip) -> MmapFn {
    scope.frames.with_current_bytecode(|bytecode| {
        fn target_from_relative(next_bc_ip: u32, rel: i16) -> Ip {
            let target = next_bc_ip as i64 + rel as i64;
            assert!(target >= 0, "computed negative bytecode ip target: {target}");
            Ip(target as u32)
        }

        let bytecode = &bytecode[start.0 as usize..end.0 as usize];

        let mut x86 = x86::Emitter::new(bytecode.len());

        // Prologue
        // START OF STACK
        x86.push(x86::Register::Rbp); // rsp aligned
        x86.mov_reg_reg(x86::Register::Rbp, x86::Register::Rsp);
        x86.push(x86::Register::R12); // Vm pointer - rbp-8, rsp misaligned by 8
        x86.mov_reg_reg(x86::Register::R12, x86::Register::Rdi);
        x86.push(x86::Register::R13); // Stub fn - rbp-16, rsp aligned
        x86.push(x86::Register::R14); // Vtable - rbp-24, rsp misaligned by 8
        x86.mov_reg_reg(x86::Register::R14, x86::Register::Rsi);
        const OUT_DATA_RBP_OFFSET: i8 = 32;
        x86.push(x86::Register::Rdx); // Out data - rbp-32, rsp aligned
        // END OF STACK

        // The stub fn is very hot, so put it in a callee-saved register
        // TODO: use a call variant that calls [r13+offset] directly
        x86.mov_reg_mem_u8(
            x86::Register::R13,
            x86::Register::Rsi,
            offset_of!(JitVtable, stub_fn).try_into().unwrap(),
        );

        // ... Body ...
        fn emit_stub_for_instr(x86: &mut x86::Emitter, instr: Instruction, ip: u32) {
            x86.mov_reg_reg(x86::Register::Rdi, x86::Register::R12);
            x86.mov_reg_mem_u8(x86::Register::Rsi, x86::Register::Rbp, -OUT_DATA_RBP_OFFSET);
            x86.mov_reg_imm32(x86::Register::Rdx, instr as u32);
            x86.mov_reg_imm32(x86::Register::Rcx, ip);
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
        assert!(start.0 + i as u32 == end.0);
        x86.mark_bytecode_ip(Ip(i as u32));
        x86.mov_reg_mem_u8(x86::Register::Rax, x86::Register::Rbp, -OUT_DATA_RBP_OFFSET);
        x86.move_mem_imm32(
            x86::Register::Rax,
            offset_of!(JitOutData, ip).try_into().unwrap(),
            end.0.cast_signed(),
        );
        x86.mov_reg_imm32(x86::Register::Eax, 0);

        // Epilogue
        x86.mark_internal_label(InternalLabel::Epilogue);
        x86.pop(x86::Register::Rdx);
        x86.pop(x86::Register::R14);
        x86.pop(x86::Register::R13);
        x86.pop(x86::Register::R12);
        x86.pop(x86::Register::Rbp);
        x86.ret();

        x86.mark_internal_label(InternalLabel::StubStatusHandler);
        x86.jmp_internal_label(InternalLabel::Epilogue);

        MmapFn::alloc(x86.buffer())
    })
}

pub fn compile_loop_region(scope: &mut LocalScope<'_>, start: Ip, end: Ip) -> JitFnHandle {
    let current_fn = Rc::as_ptr(scope.frames.current_fn());
    let key = (current_fn, start);

    if let Some(func) = scope.jit.compiled_fn_cache.get(&key) {
        JitFnHandle(Rc::clone(func))
    } else {
        let func = Rc::new(compile_uncached(scope, start, end));
        scope.jit.compiled_fn_cache.insert(key, Rc::clone(&func));
        JitFnHandle(func)
    }
}
