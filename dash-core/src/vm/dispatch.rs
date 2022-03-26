use super::{value::Value, Vm};
use crate::compiler::instruction as opcode;

pub enum HandleResult {
    Continue,
    Return(Value),
}

mod handlers {
    use crate::compiler::FunctionCallMetadata;
    use crate::vm::local::LocalScope;
    use crate::vm::value::array::Array;
    use crate::vm::value::object::NamedObject;
    use crate::vm::value::object::Object;

    use super::*;

    fn evaluate_binary_expr<F: Fn(Value, Value, &mut Vm) -> Value>(
        vm: &mut Vm,
        fun: F,
    ) -> Result<HandleResult, Value> {
        let right = vm.stack.pop().expect("No right operand");
        let left = vm.stack.pop().expect("No left operand");
        let result = fun(left, right, vm);
        vm.try_push_stack(result)?;
        Ok(HandleResult::Continue)
    }

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
        evaluate_binary_expr(vm, |l, r, _| l.add(&r))
    }

    pub fn sub(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.sub(&r))
    }

    pub fn mul(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.mul(&r))
    }

    pub fn div(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.div(&r))
    }

    pub fn rem(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.rem(&r))
    }

    pub fn pow(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.pow(&r))
    }

    pub fn lt(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.lt(&r))
    }

    pub fn le(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.le(&r))
    }

    pub fn gt(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.gt(&r))
    }

    pub fn ge(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.ge(&r))
    }

    pub fn eq(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.eq(&r))
    }

    pub fn ne(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.ne(&r))
    }

    pub fn strict_eq(vm: &mut Vm) -> Result<HandleResult, Value> {
        evaluate_binary_expr(vm, |l, r, _| l.strict_eq(&r))
    }

    pub fn pop(vm: &mut Vm) -> Result<HandleResult, Value> {
        vm.stack.pop();
        Ok(HandleResult::Continue)
    }

    pub fn ret(vm: &mut Vm) -> Result<HandleResult, Value> {
        let value = vm.stack.pop().expect("No return value");
        let this = vm.frames.pop().expect("No frame");

        unsafe {
            vm.stack.set_len(this.sp);
        };

        Ok(HandleResult::Return(value))
    }

    pub fn ldglobal(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        let constant = &vm.frames.last().expect("No frame").constants[id as usize];

        let name = constant
            .as_identifier()
            .expect("Referenced constant is not an identifier")
            .clone();

        let mut scope = LocalScope::new(vm);
        let value = scope.global.clone().get_property(&mut scope, &name)?;
        vm.stack.push(value);
        Ok(HandleResult::Continue)
    }

    pub fn call(vm: &mut Vm) -> Result<HandleResult, Value> {
        let meta = FunctionCallMetadata::from(vm.fetch_and_inc_ip());
        let argc = meta.value();
        let is_constructor = meta.is_constructor_call();
        let has_this = meta.is_object_call();

        let mut args = Vec::with_capacity(argc.into());
        let mut refs = Vec::new();
        for _ in 0..argc {
            let value = vm.stack.pop().expect("Missing argument");
            if let Value::Object(handle) = &value {
                refs.push(handle.clone());
            }

            args.push(value);
        }

        let callee = vm.stack.pop().expect("Missing callee");

        let this = if has_this {
            vm.stack.pop().expect("Missing this")
        } else {
            Value::undefined()
        };

        let mut scope = LocalScope::new(vm);
        let scoper = &scope as *const LocalScope;
        unsafe { scope.externals.add(scoper, refs) };
        let ret = callee.apply(&mut scope, this, args)?;

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

        vm.set_local(id, value.clone());
        vm.try_push_stack(value)?;

        Ok(HandleResult::Continue)
    }

    pub fn ldlocal(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        let value = vm.get_local(id as usize).expect("Invalid local reference");

        vm.try_push_stack(value)?;
        Ok(HandleResult::Continue)
    }

    pub fn arraylit(vm: &mut Vm) -> Result<HandleResult, Value> {
        let len = vm.fetch_and_inc_ip() as usize;

        let elements = vm.stack.drain(vm.stack.len() - len..).collect::<Vec<_>>();
        let array = Array::from_vec(vm, elements);
        let handle = vm.gc.register(array);
        vm.try_push_stack(Value::Object(handle))?;
        Ok(HandleResult::Continue)
    }

    pub fn objlit(vm: &mut Vm) -> Result<HandleResult, Value> {
        let len = vm.fetch_and_inc_ip() as usize;

        let elements = vm.stack.drain(vm.stack.len() - len..).collect::<Vec<_>>();

        let mut scope = LocalScope::new(vm);
        let obj = NamedObject::new(&mut scope);
        for element in elements.into_iter() {
            // Object literal constant indices are guaranteed to be 1-byte wide, for now...
            let id = scope.fetch_and_inc_ip();
            let constant = {
                let frame = scope.frames.last().expect("No frame");

                let identifier = frame.constants[id as usize]
                    .as_identifier()
                    .expect("Invalid constant reference in bytecode");

                String::from(&**identifier)
            };
            obj.set_property(&mut scope, &constant, element).unwrap();
        }

        let handle = vm.gc.register(obj);
        vm.try_push_stack(handle.into())?;

        Ok(HandleResult::Continue)
    }

    pub fn staticpropertyaccess(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        let constant = &vm.frames.last().expect("No frame").constants[id as usize];

        let ident = constant
            .as_identifier()
            .expect("Referenced constant is not an identifier")
            .clone();

        let preserve_this = vm.fetch_and_inc_ip() == 1;

        let mut scope = LocalScope::new(vm);
        // TODO: add scope to externals because calling get_property can invoke getters

        let target = if preserve_this {
            scope.stack.last().cloned()
        } else {
            scope.stack.pop()
        };

        let target = target.expect("Missing target");

        let value = target.get_property(&mut scope, &ident)?;
        vm.try_push_stack(value)?;
        Ok(HandleResult::Continue)
    }

    pub fn staticpropertyset(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        let key = vm.frames.last().expect("No frame").constants[id as usize]
            .as_identifier()
            .unwrap()
            .clone();

        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);
        target.set_property(&mut scope, &key, value.clone())?;

        vm.try_push_stack(value)?;
        Ok(HandleResult::Continue)
    }

    pub fn staticpropertysetw(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetchw_and_inc_ip();
        let key = vm.frames.last().expect("No frame").constants[id as usize]
            .as_identifier()
            .unwrap()
            .clone();

        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);
        target.set_property(&mut scope, &key, value.clone())?;

        vm.try_push_stack(value)?;
        Ok(HandleResult::Continue)
    }

    pub fn dynamicpropertyset(vm: &mut Vm) -> Result<HandleResult, Value> {
        let key = vm.stack.pop().expect("No key");
        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let key = if let Value::String(s) = key {
            s
        } else {
            todo!()
        };

        let mut scope = LocalScope::new(vm);
        target.set_property(&mut scope, &key, value.clone())?;

        vm.try_push_stack(value)?;
        Ok(HandleResult::Continue)
    }

    pub fn ldlocalext(vm: &mut Vm) -> Result<HandleResult, Value> {
        let id = vm.fetch_and_inc_ip();
        let value = vm
            .get_external(id as usize)
            .expect("Invalid local reference")
            .clone();

        vm.try_push_stack(value.into())?;
        Ok(HandleResult::Continue)
    }
}

pub fn handle(vm: &mut Vm, instruction: u8) -> Result<HandleResult, Value> {
    match instruction {
        opcode::CONSTANT => handlers::constant(vm),
        opcode::CONSTANTW => handlers::constantw(vm),
        opcode::ADD => handlers::add(vm),
        opcode::SUB => handlers::sub(vm),
        opcode::MUL => handlers::mul(vm),
        opcode::DIV => handlers::div(vm),
        opcode::REM => handlers::rem(vm),
        opcode::POW => handlers::pow(vm),
        opcode::GT => handlers::gt(vm),
        opcode::GE => handlers::ge(vm),
        opcode::LT => handlers::lt(vm),
        opcode::LE => handlers::le(vm),
        opcode::EQ => handlers::eq(vm),
        opcode::NE => handlers::ne(vm),
        opcode::STRICTEQ => handlers::strict_eq(vm),
        opcode::POP => handlers::pop(vm),
        opcode::RET => handlers::ret(vm),
        opcode::LDGLOBAL => handlers::ldglobal(vm),
        opcode::CALL => handlers::call(vm),
        opcode::JMPFALSEP => handlers::jmpfalsep(vm),
        opcode::JMP => handlers::jmp(vm),
        opcode::STORELOCAL => handlers::storelocal(vm),
        opcode::LDLOCAL => handlers::ldlocal(vm),
        opcode::ARRAYLIT => handlers::arraylit(vm),
        opcode::OBJLIT => handlers::objlit(vm),
        opcode::STATICPROPACCESS => handlers::staticpropertyaccess(vm),
        opcode::STATICPROPSET => handlers::staticpropertyset(vm),
        opcode::STATICPROPSETW => handlers::staticpropertysetw(vm),
        opcode::DYNAMICPROPSET => handlers::dynamicpropertyset(vm),
        opcode::LDLOCALEXT => handlers::ldlocalext(vm),
        _ => unimplemented!("{}", instruction),
    }
}
