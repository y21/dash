use super::{value::Value, Vm};
use crate::compiler::instruction as opcode;

pub enum HandleResult {
    Continue,
    Return(Value),
}

mod handlers {
    use crate::vm::value::array::Array;
    use crate::vm::value::object::AnonymousObject;
    use crate::vm::value::object::Object;

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
            .expect("Referenced constant is not an identifier")
            .clone();

        let value = vm.global.clone().get_property(vm, &name)?;
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
        let ret = callee.apply(vm, Value::Undefined, args)?;
        vm.try_push_stack(ret)?;
        Ok(HandleResult::Continue)
    }

    pub fn jmpfalsep(vm: &mut Vm) -> Result<HandleResult, Value> {
        let offset = vm.fetch_and_inc_ip() as i8;
        let value = vm.stack.pop().expect("No value");

        if !value.is_truthy() {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(HandleResult::Continue)
    }

    pub fn jmp(vm: &mut Vm) -> Result<HandleResult, Value> {
        let offset = vm.fetch_and_inc_ip() as i8;
        let frame = vm.frames.last_mut().expect("No frame");

        if offset.is_negative() {
            frame.ip -= -offset as usize;
        } else {
            frame.ip += offset as usize;
        }

        Ok(HandleResult::Continue)
    }

    pub fn storelocal(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip() as usize;
        let value = vm.stack.pop().expect("No value");

        vm.stack[id] = value.clone();
        vm.try_push_stack(value)?;

        Ok(HandleResult::Continue)
    }

    pub fn ldlocal(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        let value = vm.stack[id as usize].clone();

        vm.try_push_stack(value)?;
        Ok(HandleResult::Continue)
    }

    pub fn lt(vm: &mut Vm) -> Result<HandleResult, Value> {
        let right = vm.stack.pop().expect("No right operand");
        let left = vm.stack.pop().expect("No left operand");
        vm.try_push_stack(left.lt(&right))?;
        Ok(HandleResult::Continue)
    }

    pub fn arraylit(vm: &mut Vm) -> Result<HandleResult, Value> {
        let len = vm.fetch_and_inc_ip() as usize;

        let elements = vm.stack.drain(vm.stack.len() - len..).collect::<Vec<_>>();
        let array = Array::from(elements);
        let handle = vm.gc.register(array);
        vm.try_push_stack(Value::Object(handle))?;
        Ok(HandleResult::Continue)
    }

    pub fn objlit(vm: &mut Vm) -> Result<HandleResult, Value> {
        let len = vm.fetch_and_inc_ip() as usize;

        let elements = vm.stack.drain(vm.stack.len() - len..).collect::<Vec<_>>();

        let obj = AnonymousObject::new();
        for element in elements.into_iter() {
            // Object literal constant indices are guaranteed to be 1-byte wide, for now...
            let id = vm.fetch_and_inc_ip();
            let constant = {
                let frame = vm.frames.last().expect("No frame");

                let identifier = frame
                    .constants
                    .get(id as usize)
                    .expect("Invalid constant reference in bytecode")
                    .as_identifier()
                    .expect("Invalid constant reference in bytecode");

                String::from(&**identifier)
            };
            obj.set_property(vm, &constant, element).unwrap();
        }

        let handle = vm.gc.register(obj);
        vm.try_push_stack(handle.into())?;

        Ok(HandleResult::Continue)
    }

    pub fn staticpropertyaccess(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        let constant = vm
            .frames
            .last()
            .expect("No frame")
            .constants
            .get(id as usize)
            .expect("Invalid constant reference in bytecode");

        let ident = constant
            .as_identifier()
            .expect("Referenced constant is not an identifier")
            .clone();

        let target = vm.stack.pop().expect("No value");
        let value = target.get_property(vm, &ident)?;
        vm.try_push_stack(value)?;
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
        opcode::JMP => handlers::jmp(vm),
        opcode::STORELOCAL => handlers::storelocal(vm),
        opcode::LDLOCAL => handlers::ldlocal(vm),
        opcode::LT => handlers::lt(vm),
        opcode::ARRAYLIT => handlers::arraylit(vm),
        opcode::OBJLIT => handlers::objlit(vm),
        opcode::STATICPROPACCESS => handlers::staticpropertyaccess(vm),
        _ => unimplemented!("{}", instruction),
    }
}
