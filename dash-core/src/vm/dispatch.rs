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
    use std::rc::Rc;

    use crate::compiler::constant::Constant;
    use crate::compiler::FunctionCallMetadata;
    use crate::compiler::StaticImportKind;
    use crate::throw;
    use crate::vm::frame::FrameState;
    use crate::vm::frame::TryBlock;
    use crate::vm::local::LocalScope;
    use crate::vm::value::array::Array;
    use crate::vm::value::object::NamedObject;
    use crate::vm::value::object::Object;
    use crate::vm::value::object::PropertyKey;
    use crate::vm::value::ops::equality::ValueEquality;

    use super::*;

    fn force_get_constant(vm: &Vm, index: usize) -> &Constant {
        &vm.frames.last().expect("Missing frame").constants[index]
    }

    fn force_get_identifier(vm: &Vm, index: usize) -> Rc<str> {
        force_get_constant(vm, index)
            .as_identifier()
            .cloned()
            .expect("Invalid constant referenced")
    }

    fn evaluate_binary_expr<F>(vm: &mut Vm, fun: F) -> Result<Option<HandleResult>, Value>
    where
        F: Fn(&Value, &Value, &mut LocalScope) -> Result<Value, Value>,
    {
        let right = vm.stack.pop().expect("No right operand");
        let left = vm.stack.pop().expect("No left operand");
        let mut scope = LocalScope::new(vm);
        let result = fun(&left, &right, &mut scope)?;
        scope.try_push_stack(result)?;
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
        let target = vm.stack.pop().expect("Missing target");
        let source = vm.stack.pop().expect("Missing source");

        let mut sc = LocalScope::new(vm);
        sc.add_value(target.clone());
        sc.add_value(source.clone());

        let is_instanceof = source.instanceof(&target, &mut sc).map(Value::Boolean)?;
        sc.try_push_stack(is_instanceof)?;
        Ok(None)
    }

    pub fn lt(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, ValueEquality::lt)
    }

    pub fn le(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, ValueEquality::le)
    }

    pub fn gt(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, ValueEquality::gt)
    }

    pub fn ge(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, ValueEquality::ge)
    }

    pub fn eq(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, ValueEquality::eq)
    }

    pub fn ne(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, ValueEquality::ne)
    }

    pub fn strict_eq(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, ValueEquality::strict_eq)
    }

    pub fn strict_ne(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        evaluate_binary_expr(vm, ValueEquality::strict_ne)
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

        if this.is_module() {
            // Put it back on the frame stack, because we'll need it in Vm::execute_module
            vm.frames.push(this);
        }

        Ok(Some(HandleResult::Return(value)))
    }

    pub fn ldglobal(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let name = force_get_identifier(vm, id.into());

        let mut scope = LocalScope::new(vm);
        let value = scope.global.clone().get_property(&mut scope, name.as_ref().into())?;
        scope.try_push_stack(value)?;
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
        args.reverse(); // TODO: we can do better

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

        scope.try_push_stack(ret)?;
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
                let identifier = force_get_identifier(&scope, id.into());

                String::from(&*identifier)
            };

            obj.set_property(&mut scope, constant.into(), element).unwrap();
        }

        let handle = scope.gc.register(obj);
        scope.try_push_stack(handle.into())?;

        Ok(None)
    }

    pub fn staticpropertyaccess(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let ident = force_get_identifier(vm, id.into());

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
        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn staticpropertyset(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let key = force_get_identifier(vm, id.into());

        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);
        target.set_property(&mut scope, key.to_string().into(), value.clone())?;

        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn staticpropertysetw(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetchw_and_inc_ip();
        let key = force_get_identifier(vm, id.into());

        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);
        target.set_property(&mut scope, key.to_string().into(), value.clone())?;

        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn dynamicpropertyset(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let key = vm.stack.pop().expect("No key");
        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);

        let key = PropertyKey::from_value(&mut scope, key)?;
        target.set_property(&mut scope, key, value.clone())?;

        scope.try_push_stack(value)?;
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
        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn ldlocalext(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let value = vm.get_external(id as usize).expect("Invalid local reference").clone();

        vm.try_push_stack(Value::External(value))?;
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

    pub fn import_dyn(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing value");

        let ret = match vm.params.dynamic_import_callback() {
            Some(cb) => cb(vm, value)?,
            None => throw!(vm, "Dynamic imports are disabled for this context"),
        };

        // TODO: dynamic imports are currently statements, making them useless
        // TODO: make them an expression and push ret on stack

        Ok(None)
    }

    pub fn import_static(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let ty = StaticImportKind::from_repr(vm.fetch_and_inc_ip()).expect("Invalid import kind");
        let local_id = vm.fetchw_and_inc_ip();
        let path_id = vm.fetchw_and_inc_ip();

        let path = vm.frames.last().expect("No frame").constants[path_id as usize]
            .as_string()
            .expect("Referenced invalid constant")
            .clone();

        let value = match vm.params.static_import_callback() {
            Some(cb) => cb(vm, ty, &path)?,
            None => throw!(vm, "Static imports are disabled for this context."),
        };

        vm.set_local(local_id.into(), value);

        Ok(None)
    }

    pub fn export_default(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing value");
        let frame = vm.frames.last_mut().expect("Missing frame");

        match &mut frame.state {
            FrameState::Module(module) => {
                module.default = Some(value);
            }
            _ => throw!(vm, "Export is only available at the top level in modules"),
        }

        Ok(None)
    }

    pub fn export_named(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let mut sc = LocalScope::new(vm);
        let count = sc.fetchw_and_inc_ip();

        for _ in 0..count {
            let (value, ident) = match sc.fetch_and_inc_ip() {
                0 => {
                    // Local variable
                    let loc_id = sc.fetchw_and_inc_ip();
                    let ident_id = sc.fetchw_and_inc_ip();

                    let value = sc.get_local(loc_id.into()).expect("Invalid local reference");
                    let ident = force_get_identifier(&sc, ident_id.into());

                    (value, ident)
                }
                1 => {
                    // Global variable
                    let ident_id = sc.fetchw_and_inc_ip();

                    let ident = force_get_identifier(&sc, ident_id.into());

                    let global = sc.global.clone();
                    let value = global.get_property(&mut sc, ident.as_ref().into())?;

                    (value, ident)
                }
                _ => unreachable!(),
            };

            let frame = sc.frames.last_mut().expect("Missing frame");
            match &mut frame.state {
                FrameState::Module(exports) => exports.named.push((ident, value)),
                _ => throw!(&mut sc, "Export is only available at the top level in modules"),
            }
        }

        Ok(None)
    }

    pub fn debugger(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        if let Some(cb) = vm.params().debugger_callback() {
            cb(vm)?;
        }

        Ok(None)
    }

    pub fn this(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let this = vm
            .frames
            .iter()
            .rev()
            .find_map(|f| f.this.clone())
            .unwrap_or_else(|| Value::Object(vm.global.clone()));

        vm.try_push_stack(this)?;
        Ok(None)
    }

    pub fn global_this(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        vm.try_push_stack(Value::Object(vm.global.clone()))?;
        Ok(None)
    }

    pub fn super_(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        throw!(vm, "`super` keyword unexpected in this context");
    }

    pub fn revstck(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let count = vm.fetch_and_inc_ip();

        let len = vm.stack.len();
        let elements = &mut vm.stack[len - count as usize..];
        elements.reverse();

        Ok(None)
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
        opcode::IMPORTDYN => handlers::import_dyn(vm),
        opcode::IMPORTSTATIC => handlers::import_static(vm),
        opcode::EXPORTDEFAULT => handlers::export_default(vm),
        opcode::EXPORTNAMED => handlers::export_named(vm),
        opcode::THIS => handlers::this(vm),
        opcode::GLOBAL => handlers::global_this(vm),
        opcode::SUPER => handlers::super_(vm),
        opcode::DEBUGGER => handlers::debugger(vm),
        opcode::REVSTCK => handlers::revstck(vm),
        _ => unimplemented!("{}", instruction),
    }
}
