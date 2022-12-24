use std::rc::Rc;

use dash_llvm_jit_backend::legacy::assembler::JitResult;
use dash_llvm_jit_backend::passes::infer::infer_types;
use dash_llvm_jit_backend::passes::infer::InferQueryProvider;
use dash_llvm_jit_backend::passes::infer::Type;
use dash_llvm_jit_backend::Trace;
use dash_llvm_jit_backend::Value as JitValue;
use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::constant::Function;

use crate::value::Value;
use crate::Vm;

struct QueryProvider<'a> {
    vm: &'a Vm,
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
        self.vm.recording_trace.as_ref().unwrap().get_conditional_jump(nth)
    }
}

fn handle_loop_trace(vm: &mut Vm, loop_end_ip: usize) {
    let frame = vm.frames.last().unwrap();
    let trace = vm.recording_trace.as_ref().unwrap();
    let bytecode = &frame.function.buffer[trace.start()..trace.end()];

    let Ok(types) = infer_types(bytecode, QueryProvider { vm }) else {
        todo!("Mark code region as poisoned");
    };
    // println!("{x:?}");

    std::process::abort();
}

pub fn handle_loop_end(vm: &mut Vm, loop_end_ip: usize) {
    let frame = vm.frames.last().unwrap();
    let origin = Rc::as_ptr(&frame.function);

    if let Some(trace) = vm.recording_trace.as_ref() {
        if trace.start() == frame.ip {
            handle_loop_trace(vm, loop_end_ip);
        } else {
            todo!("Side exit")
        }
    } else {
        // We are jumping back to a loop header
        let frame = vm.frames.last_mut().unwrap();
        let counter = frame.loop_counter.get_or_insert(frame.ip);

        counter.inc();
        if counter.is_hot() {
            // Hot loop detected
            // Start recording a trace (i.e. every opcode) for the next loop iteration
            // The trace will go on until either:
            // - The loop is exited
            // - The iteration has ended (i.e. we are here again)
            let trace = Trace::new(origin, frame.ip, loop_end_ip, false);
            vm.recording_trace = Some(trace);
        }
    }
    // let origin = Rc::as_ptr(&frame.function);
    // let is_loop_trace = vm.recording_trace.as_ref().map_or(false, |t| t.start() == frame.ip);

    // let cache = vm.assembler.get_function(JitCacheKey {
    //     function: origin,
    //     ip: frame.ip,
    // });
    // if let Some(cache) = cache {
    //     let mut args = Vec::with_capacity(cache.locals.len());
    //     for &local in &cache.locals {
    //         args.push(match vm.get_local(local.into()).unwrap() {
    //             Value::Boolean(b) => JitValue::Boolean(b),
    //             Value::Number(Number(n)) => {
    //                 if n.floor() == n {
    //                     JitValue::Integer(n as i64)
    //                 } else {
    //                     JitValue::Number(n)
    //                 }
    //             }
    //             other => panic!("Unhandled JIT value: {:?}", other),
    //         });
    //     }

    //     let res = JitResult {
    //         function: cache.function,
    //         locals: cache.locals.clone(),
    //         values: args,
    //     };
    //     execute_jit_function(res, vm, origin, loop_end_ip);
    //     return;
    // }

    // if is_loop_trace {
    //     let trace = vm.recording_trace.take().expect("Trace must exist");

    //     let bytecode = frame.function.buffer[trace.start()..trace.end()].to_vec();
    //     let result = vm.assembler.compile_trace(trace, bytecode);
    //     execute_jit_function(result, vm, origin, loop_end_ip);
    // } else {
    //     let is_side_exit_trace = vm.recording_trace.as_ref().map_or(false, |tr| tr.side_exit());

    //     if is_side_exit_trace {
    //         let trace = vm.recording_trace.take().expect("Trace must exist");

    //         let bytecode = frame.function.buffer[trace.start()..trace.end()].to_vec();
    //         let result = vm.assembler.compile_trace(trace, bytecode);
    //         execute_jit_function(result, vm, origin, loop_end_ip);
    //     } else {
    //         // We are jumping back to a loop header
    //         let frame = vm.frames.last_mut().unwrap();
    //         let counter = frame.loop_counter.get_or_insert(frame.ip);

    //         counter.inc();
    //         if counter.is_hot() {
    //             // Hot loop detected
    //             // Start recording a trace (i.e. every opcode) for the next loop iteration
    //             // The trace will go on until either:
    //             // - The loop is exited
    //             // - The iteration has ended (i.e. we are here again)
    //             let trace = Trace::new(origin, frame.ip, loop_end_ip, false);
    //             vm.recording_trace = Some(trace);
    //         }
    //     }
    // }
}

fn execute_jit_function(mut result: JitResult, vm: &mut Vm, origin: *const Function, loop_end_ip: usize) {
    // Execute JIT function. Return value is the target instruction pointer where the VM will resume
    let ip = result.exec() as usize;

    vm.frames.last_mut().unwrap().ip = ip;

    let values = result.values.into_iter();
    let keys = result.locals.into_iter();

    for (value, local) in values.zip(keys) {
        vm.set_local(
            local as usize,
            match value {
                JitValue::Boolean(b) => Value::Boolean(b),
                JitValue::Integer(i) => Value::number(i as f64),
                JitValue::Number(n) => Value::number(n),
            },
        );
    }

    // Mark this side exit, this has the same logic as optimizing loops
    // TODO: should we be checking if the side exit is the end of the loop
    let frame = vm.frames.last_mut().unwrap();
    let counter = frame.loop_counter.get_or_insert(ip);

    counter.inc();
    if counter.is_hot() {
        let trace = Trace::new(origin, ip, loop_end_ip, true);
        vm.recording_trace = Some(trace);
    }
}
