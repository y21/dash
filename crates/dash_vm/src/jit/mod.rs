use std::rc::Rc;

mod frontend;
mod query;
pub use frontend::Frontend;
use frontend::Trace;

use crate::Vm;

fn handle_loop_trace(vm: &mut Vm, jmp_instr_ip: usize) {
    let Ok((trace, fun)) = frontend::compile_current_trace(vm) else {
        vm.poison_ip(jmp_instr_ip);
        return;
    };

    let frame_sp = {
        let frame = vm.frames.last().unwrap();
        frame.sp
    };

    let offset_ip = trace.start();
    let mut target_ip = 0;

    unsafe {
        let stack_ptr = vm.stack.as_mut_ptr().cast();
        let frame_sp = u64::try_from(frame_sp).unwrap();
        let out_target_ip = &mut target_ip;
        fun(stack_ptr, frame_sp, out_target_ip);
    }

    // `target_ip` is not the "real" IP, there may be some extra instructions before the loop header
    vm.frames.last_mut().unwrap().ip = offset_ip + target_ip as usize;
}

pub fn handle_loop_end(vm: &mut Vm, loop_end_ip: usize) {
    let frame = vm.frames.last().unwrap();
    let origin = Rc::as_ptr(&frame.function);
    let vm_instr_ip = loop_end_ip - 3;

    if let Some(trace) = vm.jit.recording_trace() {
        if trace.start() == frame.ip {
            handle_loop_trace(vm, vm_instr_ip);
        } else {
            todo!("Side exit")
        }
    } else {
        // We are jumping back to a loop header
        let frame = vm.frames.last_mut().unwrap();
        let counter = frame.loop_counter.get_or_insert(frame.ip);

        counter.inc();
        if counter.is_hot() {
            if frame.function.is_poisoned_ip(vm_instr_ip) {
                // We have already tried to compile this loop, and failed
                // So don't bother re-tracing
                return;
            }

            // Hot loop detected
            // Start recording a trace (i.e. every opcode) for the next loop iteration
            // The trace will go on until either:
            // - The loop is exited
            // - The iteration has ended (i.e. we are here again)
            let trace = Trace::new(origin, frame.ip, loop_end_ip, false);
            vm.jit.set_recording_trace(trace);
        }
    }
}
