use super::{value::Value, Vm};
use crate::compiler::instruction as opcode;

pub enum HandleResult {
    Continue,
    Return(Value),
}

mod handlers {}

pub fn handle(vm: &mut Vm, instruction: u8) -> Result<HandleResult, Value> {
    match instruction {
        opcode::CONSTANT => {
            let id = vm.fetch_and_inc_ip();
            vm.push_constant(id as usize)?;
        }
        opcode::CONSTANTW => {
            let id = vm.fetchw_and_inc_ip();
            vm.push_constant(id as usize)?;
        }
        opcode::ADD => {
            let right = vm.stack.pop().expect("No right operand");
            let left = vm.stack.pop().expect("No left operand");
            vm.try_push_stack(left.add(&right))?;
        }
        opcode::POP => {
            vm.stack.pop();
        }
        opcode::RET => {
            let value = vm.stack.pop().expect("No return value");
            let this = vm.frames.pop().expect("No frame");

            if vm.frames.is_empty() {
                // returning from the last frame means we are done
                return Ok(HandleResult::Return(value));
            }
        }
        opcode::LDGLOBAL => {
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
        }
        opcode::LDGLOBALW => {}
        opcode::CALL => {
            let argc = vm.fetch_and_inc_ip();
            let is_constructor = vm.fetch_and_inc_ip();

            let mut args = Vec::with_capacity(argc.into());
            for _ in 0..argc {
                args.push(vm.stack.pop().expect("Missing argument"));
            }

            let callee = vm.stack.pop().expect("Missing callee");
            vm.stack.push(callee.apply(Value::Undefined, args)?);
        }
        _ => unimplemented!("{}", instruction),
    }

    Ok(HandleResult::Continue)
}
