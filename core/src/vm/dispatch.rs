use super::{value::Value, Vm};
use crate::compiler::instruction as opcode;

pub enum HandleResult {
    Continue,
    Return(Value),
}

mod handlers {
    use super::*;

    pub fn constant(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        vm.push_constant(id as usize)?;
        Ok(HandleResult::Continue)
    }

    pub fn constantw(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetchw_and_inc_ip();
        vm.push_constant(id as usize)?;
        Ok(HandleResult::Continue)
    }

    pub fn add(vm: &mut Vm) -> Result<HandleResult, Value> {
        let right = vm.stack.pop().expect("No right operand");
        let left = vm.stack.pop().expect("No left operand");
        vm.try_push_stack(left.add(&right))?;
        Ok(HandleResult::Continue)
    }

    pub fn pop(vm: &mut Vm) -> Result<HandleResult, Value> {
        vm.stack.pop();
        Ok(HandleResult::Continue)
    }

    pub fn ret(vm: &mut Vm) -> Result<HandleResult, Value> {
        let value = vm.stack.pop().expect("No return value");
        let this = vm.frames.pop().expect("No frame");

        if vm.frames.is_empty() {
            // returning from the last frame means we are done
            Ok(HandleResult::Return(value))
        } else {
            todo!()
        }
    }

    pub fn ldglobal(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        let constant = vm
            .frames
            .last()
            .expect("No frame")
            .constants
            .get(id as usize)
            .expect("Invalid constant reference in bytecode");

        let name = constant
            .as_identifier()
            .expect("Referenced constant is not an identifier");

        let value = vm.global.get_property(name)?;
        vm.stack.push(value);
        Ok(HandleResult::Continue)
    }

    pub fn call(vm: &mut Vm) -> Result<HandleResult, Value> {
        let argc = vm.fetch_and_inc_ip();
        let is_constructor = vm.fetch_and_inc_ip();

        let mut args = Vec::with_capacity(argc.into());
        for _ in 0..argc {
            args.push(vm.stack.pop().expect("Missing argument"));
        }

        let callee = vm.stack.pop().expect("Missing callee");
        vm.stack.push(callee.apply(Value::Undefined, args)?);
        Ok(HandleResult::Continue)
    }

    pub fn jmpfalsep(vm: &mut Vm) -> Result<HandleResult, Value> {
        let offset = vm.fetch_and_inc_ip() as i8;
        let value = vm.stack.pop().expect("No value");

        if !value.is_truthy() {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= offset as usize;
            } else {
                frame.ip += offset as usize;
            }

            println!("Jumped to {}", frame.buffer[frame.ip]);
        }

        Ok(HandleResult::Continue)
    }
}

pub fn handle(vm: &mut Vm, instruction: u8) -> Result<HandleResult, Value> {
    match instruction {
        opcode::CONSTANT => handlers::constant(vm),
        opcode::CONSTANTW => handlers::constantw(vm),
        opcode::ADD => handlers::add(vm),
        opcode::POP => handlers::pop(vm),
        opcode::RET => handlers::ret(vm),
        opcode::LDGLOBAL => handlers::ldglobal(vm),
        opcode::CALL => handlers::call(vm),
        opcode::JMPFALSEP => handlers::jmpfalsep(vm),
        _ => unimplemented!("{}", instruction),
    }
}
