use std::ffi::c_void;
use std::mem;
use std::ptr::{self, NonNull};

use dash_middle::compiler::instruction::Instruction;

use crate::Vm;
use crate::dispatch::{DispatchContext, INSTRUCTION_LUT};
use crate::frame::Ip;
use crate::localscope::LocalScope;
use crate::value::Unrooted;

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
}

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
    println!("{start:x?}..{end:x?}");
    let func = scope.frames.with_current_bytecode(|bytecode| {
        let bytecode = &bytecode[start.0 as usize..end.0 as usize];

        let mut x86 = x86::Emitter::new();

        // Prologue
        x86.push(x86::Register::Rbp);
        x86.mov_reg_reg(x86::Register::Rbp, x86::Register::Rsp);
        x86.push(x86::Register::R12);
        x86.mov_reg_reg(x86::Register::R12, x86::Register::Rdi);
        x86.push(x86::Register::R13);
        // Move the stub handler into r13
        x86.mov_reg_mem_u8(x86::Register::R13, x86::Register::Rsi, 0);

        // ... Body ...

        let mut i = 0;
        while i < bytecode.len() {
            let instr = Instruction::from_repr(bytecode[i]).unwrap();
            let ip = start.0 + i as u32 + 1;
            dbg!(instr);

            x86.mov_reg_reg(x86::Register::Rdi, x86::Register::R12);
            x86.mov_reg_imm32(x86::Register::Rsi, instr as u32);
            x86.mov_reg_imm32(x86::Register::Rdx, ip);
            x86.call_reg(x86::Register::R13);

            break; // Just handle one op for now
        }

        // Epilogue
        x86.pop(x86::Register::R13);
        x86.pop(x86::Register::R12);
        x86.pop(x86::Register::Rbp);
        x86.ret();

        mmap_jit_fn(x86.buffer())
    });

    println!("Compiled! {func:?}");

    let vtable = JitVtable { stub_fn: handler_stub };
    let res = unsafe { func.call2::<&mut Vm, &JitVtable, ()>(scope, &vtable) };
    println!("Worked?");
}

#[test]
fn test_x86() {
    let mut x86 = x86::Emitter::new();
    x86.mov_reg_mem_u8(x86::Register::R13, x86::Register::Rsi, 0);
    println!("{:x?}", x86.buffer());
}
