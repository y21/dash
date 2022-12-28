use std::rc::Rc;

use dash_llvm_jit_backend::function::CompileQuery;
use dash_llvm_jit_backend::function::JITConstant;
use dash_llvm_jit_backend::passes::infer::infer_types_and_labels;
use dash_llvm_jit_backend::passes::infer::InferQueryProvider;
use dash_llvm_jit_backend::passes::infer::Type;
use dash_llvm_jit_backend::Trace;
use dash_middle::compiler::constant::Constant;

use crate::value::Value;
use crate::Vm;

struct QueryProvider<'a> {
    vm: &'a Vm,
    trace: &'a Trace,
}

impl<'a> InferQueryProvider for QueryProvider<'a> {
    fn type_of_constant(&self, index: u16) -> Option<Type> {
        let constant = &self.vm.frames.last().unwrap().function.constants[usize::from(index)];
        match constant {
            Constant::Boolean(..) => Some(Type::Boolean),
            Constant::Number(..) => Some(Type::F64),
            _ => None,
        }
    }
    fn type_of_local(&self, index: u16) -> Option<Type> {
        match self.vm.get_local(index.into()).unwrap() {
            Value::Boolean(..) => Some(Type::Boolean),
            Value::Number(..) => Some(Type::F64),
            _ => None,
        }
    }
    fn did_take_nth_branch(&self, nth: usize) -> bool {
        self.trace.get_conditional_jump(nth)
    }
}

impl<'a> CompileQuery for QueryProvider<'a> {
    fn get_constant(&self, id: u16) -> JITConstant {
        let constant = &self.vm.frames.last().unwrap().function.constants[usize::from(id)];
        match constant {
            Constant::Boolean(b) => JITConstant::Boolean(*b),
            Constant::Number(n) => JITConstant::F64(*n), // TODO: I64 may be ok
            _ => todo!(),
        }
    }
}

fn handle_loop_trace(vm: &mut Vm, jmp_instr_ip: usize) {
    let frame = vm.frames.last().unwrap();
    let trace = vm.recording_trace.take().unwrap();
    let bytecode = &frame.function.buffer[trace.start()..trace.end()];

    let Ok(types) = infer_types_and_labels(bytecode, QueryProvider { vm, trace: &trace }) else {
        vm.poison_ip(jmp_instr_ip);
        return;
    };

    let fun = vm
        .jit_backend
        .compile_trace(QueryProvider { vm, trace: &trace }, bytecode, types, &trace);

    unsafe {
        fun(vm.stack.as_mut_ptr().cast(), u64::try_from(frame.sp).unwrap());
    }

    // TODO (important): synchronize frame ip here, depending on the exit
}

pub fn handle_loop_end(vm: &mut Vm, loop_end_ip: usize) {
    let frame = vm.frames.last().unwrap();
    let origin = Rc::as_ptr(&frame.function);
    let vm_instr_ip = loop_end_ip - 3;

    if let Some(trace) = vm.recording_trace.as_ref() {
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
            vm.recording_trace = Some(trace);
        }
    }
}
