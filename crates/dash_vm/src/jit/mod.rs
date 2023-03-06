use std::rc::Rc;

mod frontend;
mod query;
use dash_log::debug;
use dash_log::error;
use dash_log::warn;
pub use frontend::Frontend;
use frontend::Trace;

use crate::Vm;

fn handle_loop_trace(vm: &mut Vm, jmp_instr_ip: usize) {
    debug!("end of loop tracing");
    let (trace, fun) = match frontend::compile_current_trace(vm) {
        Ok(t) => t,
        Err(err) => {
            error!("JIT compilation failed! {err:?}");
            vm.poison_ip(jmp_instr_ip);
            return;
        }
    };

    let frame_sp = {
        let frame = vm.frames.last().unwrap();
        frame.sp
    };

    let offset_ip = trace.start();
    let mut target_ip = 0;

    debug!("call into jit");
    unsafe {
        let stack_ptr = vm.stack.as_mut_ptr().cast();
        let frame_sp = u64::try_from(frame_sp).unwrap();
        let out_target_ip = &mut target_ip;
        fun(stack_ptr, frame_sp, out_target_ip);
    }

    target_ip = offset_ip as u64 + target_ip;
    debug!("jit returned");
    debug!(target_ip);

    // `target_ip` is not the "real" IP, there may be some extra instructions before the loop header
    vm.frames.last_mut().unwrap().ip = target_ip as usize;
}

pub fn handle_loop_end(vm: &mut Vm, loop_end_ip: usize) {
    let frame = vm.frames.last_mut().unwrap();

    // We are jumping back to a loop header
    if let Some(trace) = vm.jit.recording_trace() {
        // There is a trace being recorded for this loop
        if frame.ip == trace.start() {
            handle_loop_trace(vm, loop_end_ip);
        } else {
            todo!("Side exit");
        }
    } else {
        handle_loop_counter_inc(vm, loop_end_ip, None);
    }
}

fn handle_loop_counter_inc(vm: &mut Vm, loop_end_ip: usize, parent_ip: Option<usize>) {
    let frame = vm.frames.last_mut().unwrap();
    let origin = Rc::as_ptr(&frame.function);
    let counter = frame.loop_counter.get_or_insert(frame.ip);

    counter.inc();
    if counter.is_hot() {
        if frame.function.is_poisoned_ip(loop_end_ip) {
            // We have already tried to compile this loop, and failed
            // So don't bother re-tracing
            warn!("loop is poisoned, cannot jit");
            return;
        }

        // Hot loop detected
        // Start recording a trace (i.e. every opcode) for the next loop iteration
        // The trace will go on until either:
        // - The loop is exited
        // - The iteration has ended (i.e. we are here again)
        debug!("detected hot loop, begin trace");
        debug!(loop_end_ip, parent_ip);
        let trace = Trace::new(origin, frame.ip, loop_end_ip);
        vm.jit.set_recording_trace(trace);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dash_compiler::FunctionCompiler;
    use dash_llvm_jit_backend::passes::bb_generation::find_labels;
    use dash_llvm_jit_backend::passes::bb_generation::BBGenerationCtxt;
    use dash_optimizer::OptLevel;

    #[test]
    pub fn llvm() {
        let cr = FunctionCompiler::compile_str(
            r"

        for (let i = 0; i < 10; i++) {
            let x = 3;
        }
        ",
            OptLevel::None,
        )
        .unwrap();
        let bytecode = &cr.instructions;

        let labels = find_labels(bytecode).unwrap();

        let mut bcx = BBGenerationCtxt {
            bytecode,
            labels: labels.0,
            bbs: HashMap::new(),
        };
        bcx.find_bbs();
        bcx.resolve_edges();
        dbg!(bcx);
    }
}
