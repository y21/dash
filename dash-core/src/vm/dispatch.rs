use super::{value::Value, Vm};
use crate::compiler::instruction as opcode;

pub enum HandleResult {
    Return(Value),
    Yield(Value),
}

impl HandleResult {
    pub fn into_value(self) -> Value {
        match self {
            HandleResult::Return(v) => v,
            HandleResult::Yield(v) => v,
        }
    }
}

mod handlers {
    use crate::compiler::FunctionCallMetadata;
    use crate::vm::frame::TryBlock;
    use crate::vm::local::LocalScope;
    use crate::vm::value::array::Array;
    use crate::vm::value::object::NamedObject;
    use crate::vm::value::object::Object;
    use crate::vm::value::object::PropertyKey;

    use super::*;

    fn evaluate_binary_expr<F>(vm: &mut Vm, fun: F) -> Result<Option<HandleResult>, Value>
    where
        F: Fn(&Value, &Value, &mut LocalScope) -> Result<Value, Value>,
    {
        let right = vm.stack.pop().expect("No right operand");
        let left = vm.stack.pop().expect("No left operand");
        let mut scope = LocalScope::new(vm);
        let result = fun(&left, &right, &mut scope)?;
        vm.try_push_stack(result)?;
        Ok(None)
    }

    pub fn constant(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        vm.push_constant(id as usize)?;
        Ok(None)
    }

    pub fn constantw(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetchw_and_inc_ip();
        vm.push_constant(id as usize)?;
        Ok(None)
    }

    pub fn add(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::add)
    }

    pub fn sub(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::sub)
    }

    pub fn mul(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::mul)
    }

    pub fn div(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::div)
    }

    pub fn rem(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::rem)
    }

    pub fn pow(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::pow)
    }

    pub fn bitor(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::bitor)
    }

    pub fn bitxor(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::bitxor)
    }

    pub fn bitand(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::bitand)
    }

    pub fn bitshl(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::bitshl)
    }

    pub fn bitshr(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::bitshr)
    }

    pub fn bitushr(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, Value::bitushr)
    }

    pub fn objin(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        todo!()
    }

    pub fn instanceof(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        todo!()
    }

    pub fn lt(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, |l, r, _| Ok(l.lt(&r)))
    }

    pub fn le(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, |l, r, _| Ok(l.le(&r)))
    }

    pub fn gt(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, |l, r, _| Ok(l.gt(&r)))
    }

    pub fn ge(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, |l, r, _| Ok(l.ge(&r)))
    }

    pub fn eq(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, |l, r, _| Ok(l.eq(&r)))
    }

    pub fn ne(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, |l, r, _| Ok(l.ne(&r)))
    }

    pub fn strict_eq(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, |l, r, _| Ok(l.strict_eq(&r)))
    }

    pub fn strict_ne(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, |l, r, _| Ok(l.strict_ne(&r)))
    }

    pub fn not(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("No operand");
        let result = value.not();
        vm.try_push_stack(result)?;
        Ok(None)
    }

    pub fn pop(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        vm.stack.pop();
        Ok(None)
    }

    pub fn ret(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("No return value");
        let this = vm.frames.pop().expect("No frame");

        drop(vm.stack.drain(this.sp..));

        Ok(Some(HandleResult::Return(value)))
    }

    pub fn ldglobal(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let constant = &vm.frames.last().expect("No frame").constants[id as usize];

        let name = constant
            .as_identifier()
            .expect("Referenced constant is not an identifier")
            .clone();

        let mut scope = LocalScope::new(vm);
        let value = scope.global.clone().get_property(&mut scope, name.as_ref().into())?;
        vm.stack.push(value);
        Ok(None)
    }

    pub fn call(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
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
        Ok(None)
    }

    pub fn jmpfalsep(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let value = vm.stack.pop().expect("No value");

        if !value.is_truthy() {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmpfalsenp(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let value = vm.stack.last().expect("No value");

        if !value.is_truthy() {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmptruep(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let value = vm.stack.pop().expect("No value");

        if value.is_truthy() {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmptruenp(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let value = vm.stack.last().expect("No value");

        if value.is_truthy() {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmpnullishp(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let value = vm.stack.pop().expect("No value");

        if value.is_nullish() {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmpnullishnp(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let value = vm.stack.last().expect("No value");

        if value.is_nullish() {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmp(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let frame = vm.frames.last_mut().expect("No frame");

        if offset.is_negative() {
            frame.ip -= -offset as usize;
        } else {
            frame.ip += offset as usize;
        }

        Ok(None)
    }

    pub fn storelocal(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip() as usize;
        let value = vm.stack.pop().expect("No value");

        vm.set_local(id, value.clone());
        vm.try_push_stack(value)?;

        Ok(None)
    }

    pub fn ldlocal(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let value = vm.get_local(id as usize).expect("Invalid local reference");

        vm.try_push_stack(value)?;
        Ok(None)
    }

    pub fn arraylit(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let len = vm.fetch_and_inc_ip() as usize;

        let elements = vm.stack.drain(vm.stack.len() - len..).collect::<Vec<_>>();
        let array = Array::from_vec(vm, elements);
        let handle = vm.gc.register(array);
        vm.try_push_stack(Value::Object(handle))?;
        Ok(None)
    }

    pub fn objlit(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
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

            obj.set_property(&mut scope, constant.into(), element).unwrap();
        }

        let handle = vm.gc.register(obj);
        vm.try_push_stack(handle.into())?;

        Ok(None)
    }

    pub fn staticpropertyaccess(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
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

        let value = target.get_property(&mut scope, ident.as_ref().into())?;
        vm.try_push_stack(value)?;
        Ok(None)
    }

    pub fn staticpropertyset(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let key = vm.frames.last().expect("No frame").constants[id as usize]
            .as_identifier()
            .unwrap()
            .clone();

        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);
        target.set_property(&mut scope, key.to_string().into(), value.clone())?;

        vm.try_push_stack(value)?;
        Ok(None)
    }

    pub fn staticpropertysetw(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetchw_and_inc_ip();
        let key = vm.frames.last().expect("No frame").constants[id as usize]
            .as_identifier()
            .unwrap()
            .clone();

        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);
        target.set_property(&mut scope, key.to_string().into(), value.clone())?;

        vm.try_push_stack(value)?;
        Ok(None)
    }

    pub fn dynamicpropertyset(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let key = vm.stack.pop().expect("No key");
        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);

        let key = PropertyKey::from_value(&mut scope, key)?;
        target.set_property(&mut scope, key, value.clone())?;

        vm.try_push_stack(value)?;
        Ok(None)
    }

    pub fn dynamicpropertyaccess(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let key = vm.stack.pop().expect("No key");

        let preserve_this = vm.fetch_and_inc_ip() == 1;

        let mut scope = LocalScope::new(vm);
        // TODO: add scope to externals because calling get_property can invoke getters

        let target = if preserve_this {
            scope.stack.last().cloned()
        } else {
            scope.stack.pop()
        };

        let target = target.expect("Missing target");

        let key = PropertyKey::from_value(&mut scope, key)?;

        let value = target.get_property(&mut scope, key)?;
        vm.try_push_stack(value)?;
        Ok(None)
    }

    pub fn ldlocalext(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let value = vm.get_external(id as usize).expect("Invalid local reference").clone();

        vm.try_push_stack(value.into())?;
        Ok(None)
    }

    pub fn storelocalext(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let value = vm.stack.pop().expect("No value");

        let external = vm.frames.last_mut().expect("No frame").externals[id as usize].as_ptr();
        // TODO: make sure that nothing really aliases this &mut
        unsafe { (*external).value = value.clone().into_boxed() };

        vm.try_push_stack(value)?;

        Ok(None)
    }

    pub fn try_block(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let ip = vm.frames.last().unwrap().ip;
        let catch_offset = vm.fetchw_and_inc_ip() as usize;
        let catch_ip = ip + catch_offset + 2;

        vm.try_blocks.push(TryBlock {
            catch_ip,
            frame_ip: vm.frames.len(),
        });

        Ok(None)
    }

    pub fn try_end(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        vm.try_blocks.pop();
        Ok(None)
    }

    pub fn throw(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        Err(vm.stack.pop().expect("Missing value"))
    }

    pub fn type_of(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing value");
        vm.try_push_stack(value.type_of().as_value(vm))?;
        Ok(None)
    }

    pub fn yield_(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing value");
        Ok(Some(HandleResult::Yield(value)))
    }
}

pub fn handle(vm: &mut Vm, instruction: u8) -> Result<Option<HandleResult>, Value> {
    match instruction {
        opcode::CONSTANT => handlers::constant(vm),
        opcode::CONSTANTW => handlers::constantw(vm),
        opcode::ADD => handlers::add(vm),
        opcode::SUB => handlers::sub(vm),
        opcode::MUL => handlers::mul(vm),
        opcode::DIV => handlers::div(vm),
        opcode::REM => handlers::rem(vm),
        opcode::POW => handlers::pow(vm),
        opcode::BITOR => handlers::bitor(vm),
        opcode::BITXOR => handlers::bitxor(vm),
        opcode::BITAND => handlers::bitand(vm),
        opcode::BITSHL => handlers::bitshl(vm),
        opcode::BITSHR => handlers::bitshr(vm),
        opcode::BITUSHR => handlers::bitushr(vm),
        opcode::OBJIN => handlers::objin(vm),
        opcode::INSTANCEOF => handlers::instanceof(vm),
        opcode::GT => handlers::gt(vm),
        opcode::GE => handlers::ge(vm),
        opcode::LT => handlers::lt(vm),
        opcode::LE => handlers::le(vm),
        opcode::EQ => handlers::eq(vm),
        opcode::NE => handlers::ne(vm),
        opcode::STRICTEQ => handlers::strict_eq(vm),
        opcode::STRICTNE => handlers::strict_ne(vm),
        opcode::NOT => handlers::not(vm),
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
        opcode::DYNAMICPROPACCESS => handlers::dynamicpropertyaccess(vm),
        opcode::LDLOCALEXT => handlers::ldlocalext(vm),
        opcode::STORELOCALEXT => handlers::storelocalext(vm),
        opcode::TRY => handlers::try_block(vm),
        opcode::TRYEND => handlers::try_end(vm),
        opcode::THROW => handlers::throw(vm),
        opcode::TYPEOF => handlers::type_of(vm),
        opcode::YIELD => handlers::yield_(vm),
        opcode::JMPFALSENP => handlers::jmpfalsenp(vm),
        opcode::JMPTRUEP => handlers::jmptruep(vm),
        opcode::JMPTRUENP => handlers::jmptruenp(vm),
        opcode::JMPNULLISHP => handlers::jmpnullishp(vm),
        opcode::JMPNULLISHNP => handlers::jmpnullishnp(vm),
        _ => unimplemented!("{}", instruction),
    }
}
