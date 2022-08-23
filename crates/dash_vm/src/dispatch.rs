use super::{value::Value, Vm};
use dash_middle::compiler::instruction as inst;

pub enum HandleResult {
    Return(Value),
    Yield(Value),
    Await(Value),
}

impl HandleResult {
    pub fn into_value(self) -> Value {
        match self {
            HandleResult::Return(v) => v,
            HandleResult::Yield(v) => v,
            HandleResult::Await(v) => v,
        }
    }
}

mod handlers {
    use dash_middle::compiler::constant::Constant;
    use dash_middle::compiler::FunctionCallMetadata;
    use dash_middle::compiler::ObjectMemberKind;
    use dash_middle::compiler::StaticImportKind;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::rc::Rc;

    use crate::frame::Frame;
    use crate::frame::FrameState;
    use crate::frame::TryBlock;
    use crate::local::LocalScope;
    use crate::throw;
    use crate::value::array::Array;
    use crate::value::object::NamedObject;
    use crate::value::object::Object;
    use crate::value::object::PropertyKey;
    use crate::value::object::PropertyValue;
    use crate::value::object::PropertyValueKind;
    use crate::value::ops::abstractions::conversions::ValueConversion;
    use crate::value::ops::equality::ValueEquality;

    use super::*;

    fn force_get_constant(vm: &Vm, index: usize) -> &Constant {
        &vm.frames.last().expect("Missing frame").function.constants[index]
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

    fn constant_instruction(vm: &mut Vm, idx: usize) -> Result<(), Value> {
        let frame = vm.frames.last().expect("No frame");
        let constant = frame.function.constants[idx].clone();

        #[cfg(feature = "jit")]
        vm.record_constant(idx as u16, &constant);

        let value = Value::from_constant(constant, vm);
        vm.try_push_stack(value)?;
        Ok(())
    }

    pub fn constant(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        constant_instruction(vm, id as usize)?;
        Ok(None)
    }

    pub fn constantw(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetchw_and_inc_ip();
        constant_instruction(vm, id as usize)?;
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

    pub fn bitnot(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing value");
        let mut sc = LocalScope::new(vm);
        let result = value.bitnot(&mut sc)?;
        sc.try_push_stack(result)?;
        Ok(None)
    }

    pub fn objin(_vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
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

    pub fn neg(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing operand");
        let mut scope = LocalScope::new(vm);
        let result = value.to_number(&mut scope)?;
        scope.try_push_stack(Value::Number(-result))?;
        Ok(None)
    }

    pub fn pos(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing operand");
        let mut scope = LocalScope::new(vm);
        let result = value.to_number(&mut scope)?;
        scope.try_push_stack(Value::Number(result))?;
        Ok(None)
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
        let tc_depth = vm.fetchw_and_inc_ip();
        let value = vm.stack.pop().expect("No return value");

        let this = vm.frames.pop().expect("No frame");

        // Drain all try catch blocks that are in this frame.
        let lower_tcp = vm.try_blocks.len() - usize::from(tc_depth);
        drop(vm.try_blocks.drain(lower_tcp..));

        // Drain all the stack space from this frame
        drop(vm.stack.drain(this.sp..));

        match this.state {
            FrameState::Module(_) => {
                // Put it back on the frame stack, because we'll need it in Vm::execute_module
                vm.frames.push(this)
            }
            FrameState::Function { is_constructor_call } => {
                // If this is a constructor call and the return value is not an object,
                // return `this`
                if is_constructor_call && !matches!(value, Value::Object(_) | Value::External(_)) {
                    if let Frame { this: Some(this), .. } = this {
                        return Ok(Some(HandleResult::Return(this)));
                    }
                }
            }
        }

        Ok(Some(HandleResult::Return(value)))
    }

    pub fn ldglobal(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let name = force_get_identifier(vm, id.into());

        let mut scope = LocalScope::new(vm);

        let value = match scope.global.as_any().downcast_ref::<NamedObject>() {
            Some(value) => match value.get_raw_property(name.as_ref().into()) {
                Some(value) => value.kind().get_or_apply(&mut scope, Value::undefined())?,
                None => throw!(&mut scope, "{} is not defined", name),
            },
            None => scope.global.clone().get_property(&mut scope, name.as_ref().into())?,
        };

        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn storeglobal(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let name = force_get_identifier(vm, id.into());
        let value = vm.stack.pop().expect("No value");

        let mut scope = LocalScope::new(vm);
        scope.global.clone().set_property(
            &mut scope,
            ToString::to_string(&name).into(),
            PropertyValue::static_default(value.clone()),
        )?;
        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn call(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let meta = FunctionCallMetadata::from(vm.fetch_and_inc_ip());
        let argc = meta.value();
        let is_constructor = meta.is_constructor_call();
        let has_this = meta.is_object_call();

        let (args, refs) = {
            let argc = argc.into();
            let mut args = Vec::with_capacity(argc);
            let mut refs = Vec::new();

            let len = vm.stack.len();
            let iter = vm.stack.drain((len - argc)..);

            for value in iter {
                if let Value::Object(handle) = &value {
                    refs.push(handle.clone());
                }

                args.push(value);
            }

            (args, refs)
        };

        let callee = vm.stack.pop().expect("Missing callee");

        let this = if has_this {
            vm.stack.pop().expect("Missing this")
        } else {
            Value::undefined()
        };

        let mut scope = LocalScope::new(vm);
        let scoper = &scope as *const LocalScope;
        scope.externals.add(scoper, refs);

        let ret = if is_constructor {
            callee.construct(&mut scope, this, args)?
        } else {
            callee.apply(&mut scope, this, args)?
        };

        scope.try_push_stack(ret)?;
        Ok(None)
    }

    pub fn jmpfalsep(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let value = vm.stack.pop().expect("No value");

        let jump = !value.is_truthy();

        #[cfg(feature = "jit")]
        vm.record_conditional_jump(jump);

        if jump {
            let frame = vm.frames.last_mut().expect("No frame");

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;

                // let is_trace = vm.recording_trace.as_ref().map_or(false, |t| t.end() == frame.ip);

                // if is_trace {
                //     let _trace = vm.recording_trace.take().expect("Trace must exist");
                //     // println!("end of trace, ip {:?}", trace);
                // }
            }
        }

        Ok(None)
    }

    pub fn jmpfalsenp(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let offset = vm.fetchw_and_inc_ip() as i16;
        let value = vm.stack.last().expect("No value");

        let jump = !value.is_truthy();

        #[cfg(feature = "jit")]
        vm.record_conditional_jump(jump);

        if jump {
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

        let jump = value.is_truthy();

        #[cfg(feature = "jit")]
        vm.record_conditional_jump(jump);

        if jump {
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

        let jump = value.is_truthy();

        #[cfg(feature = "jit")]
        vm.record_conditional_jump(jump);

        if jump {
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

        let jump = value.is_nullish();

        #[cfg(feature = "jit")]
        vm.record_conditional_jump(jump);

        if jump {
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

        let jump = value.is_nullish();

        #[cfg(feature = "jit")]
        vm.record_conditional_jump(jump);

        if jump {
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

        // Note: this is an unconditional jump, so we don't push this into the trace as a conditional jump

        if offset.is_negative() {
            #[cfg(feature = "jit")]
            let old_ip = frame.ip;

            frame.ip -= -offset as usize;

            // Negative jumps are (currently) always also a marker for the end of a loop
            // and we want to JIT compile loops that run often
            #[cfg(feature = "jit")]
            crate::jit::handle_loop_end(vm, old_ip);
        } else {
            frame.ip += offset as usize;
        }

        Ok(None)
    }

    pub fn storelocal(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip() as usize;
        let value = vm.stack.pop().expect("No value");

        #[cfg(feature = "jit")]
        vm.record_local(id as u16, &value);

        vm.set_local(id, value.clone());
        vm.try_push_stack(value)?;

        Ok(None)
    }

    pub fn ldlocal(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetch_and_inc_ip();
        let value = vm.get_local(id as usize).expect("Invalid local reference");

        #[cfg(feature = "jit")]
        vm.record_local(id as u16, &value);

        vm.try_push_stack(value)?;
        Ok(None)
    }

    pub fn arraylit(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let len = vm.fetch_and_inc_ip() as usize;

        let elements = vm
            .stack
            .drain(vm.stack.len() - len..)
            .map(PropertyValue::static_default)
            .collect::<Vec<_>>();
        let array = Array::from_vec(vm, elements);
        let handle = vm.gc.register(array);
        vm.try_push_stack(Value::Object(handle))?;
        Ok(None)
    }

    pub fn objlit(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let len = vm.fetch_and_inc_ip() as usize;

        let mut scope = LocalScope::new(vm);
        let mut obj = HashMap::new();
        for _ in 0..len {
            let value = scope.stack.pop().unwrap();
            let kind = ObjectMemberKind::from_repr(scope.fetch_and_inc_ip()).unwrap();

            let key = match kind {
                // TODO: it might be a symbol, don't to_string it then!
                ObjectMemberKind::Dynamic => {
                    let key = scope.stack.pop().unwrap().to_string(&mut scope)?;
                    PropertyKey::String(Cow::Owned(String::from(&*key)))
                }
                ObjectMemberKind::Getter | ObjectMemberKind::Setter | ObjectMemberKind::Static => {
                    let id = scope.fetch_and_inc_ip();

                    // TODO: optimization opportunity: do not reallocate string from Rc<str>
                    let key = String::from(&*force_get_identifier(&scope, id.into()));
                    PropertyKey::String(Cow::Owned(key))
                }
            };

            match kind {
                ObjectMemberKind::Dynamic | ObjectMemberKind::Static => {
                    drop(obj.insert(key, PropertyValue::static_default(value)))
                }
                ObjectMemberKind::Getter => {
                    let value = match value {
                        Value::Object(o) => o,
                        _ => panic!("Getter is not an object"),
                    };

                    obj.entry(key)
                        .and_modify(|v| match v.kind_mut() {
                            PropertyValueKind::Trap { get, .. } => {
                                *get = Some(value.clone());
                            }
                            _ => *v = PropertyValue::getter_default(value.clone()),
                        })
                        .or_insert_with(|| PropertyValue::getter_default(value));
                }
                ObjectMemberKind::Setter => {
                    let value = match value {
                        Value::Object(o) => o,
                        _ => panic!("Setter is not an object"),
                    };

                    obj.entry(key)
                        .and_modify(|v| match v.kind_mut() {
                            PropertyValueKind::Trap { set, .. } => {
                                *set = Some(value.clone());
                            }
                            _ => *v = PropertyValue::setter_default(value.clone()),
                        })
                        .or_insert_with(|| PropertyValue::setter_default(value));
                }
            };
        }

        let obj = NamedObject::with_values(&mut scope, obj);

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
        target.set_property(
            &mut scope,
            ToString::to_string(&key).into(),
            PropertyValue::static_default(value.clone()),
        )?;

        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn staticpropertysetw(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let id = vm.fetchw_and_inc_ip();
        let key = force_get_identifier(vm, id.into());

        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);
        target.set_property(
            &mut scope,
            ToString::to_string(&key).into(),
            PropertyValue::static_default(value.clone()),
        )?;

        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn dynamicpropertyset(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let key = vm.stack.pop().expect("No key");
        let value = vm.stack.pop().expect("No value");
        let target = vm.stack.pop().expect("No target");

        let mut scope = LocalScope::new(vm);

        let key = PropertyKey::from_value(&mut scope, key)?;
        target.set_property(&mut scope, key, PropertyValue::static_default(value.clone()))?;

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
        vm.try_push_stack(value.type_of().as_value())?;
        Ok(None)
    }

    pub fn yield_(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing value");
        Ok(Some(HandleResult::Yield(value)))
    }

    pub fn await_(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing value");
        Ok(Some(HandleResult::Await(value)))
    }

    pub fn import_dyn(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        let value = vm.stack.pop().expect("Missing value");

        let _ret = match vm.params.dynamic_import_callback() {
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

        let path = vm.frames.last().expect("No frame").function.constants[path_id as usize]
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

    pub fn undef(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        vm.try_push_stack(Value::undefined())?;
        Ok(None)
    }

    pub fn infinity(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        vm.try_push_stack(Value::Number(f64::INFINITY))?;
        Ok(None)
    }

    pub fn nan(vm: &mut Vm) -> Result<Option<HandleResult>, Value> {
        vm.try_push_stack(Value::Number(f64::NAN))?;
        Ok(None)
    }
}

pub fn handle(vm: &mut Vm, instruction: u8) -> Result<Option<HandleResult>, Value> {
    match instruction {
        inst::CONSTANT => handlers::constant(vm),
        inst::CONSTANTW => handlers::constantw(vm),
        inst::ADD => handlers::add(vm),
        inst::SUB => handlers::sub(vm),
        inst::MUL => handlers::mul(vm),
        inst::DIV => handlers::div(vm),
        inst::REM => handlers::rem(vm),
        inst::POW => handlers::pow(vm),
        inst::BITOR => handlers::bitor(vm),
        inst::BITXOR => handlers::bitxor(vm),
        inst::BITAND => handlers::bitand(vm),
        inst::BITSHL => handlers::bitshl(vm),
        inst::BITSHR => handlers::bitshr(vm),
        inst::BITUSHR => handlers::bitushr(vm),
        inst::BITNOT => handlers::bitnot(vm),
        inst::OBJIN => handlers::objin(vm),
        inst::INSTANCEOF => handlers::instanceof(vm),
        inst::GT => handlers::gt(vm),
        inst::GE => handlers::ge(vm),
        inst::LT => handlers::lt(vm),
        inst::LE => handlers::le(vm),
        inst::EQ => handlers::eq(vm),
        inst::NE => handlers::ne(vm),
        inst::STRICTEQ => handlers::strict_eq(vm),
        inst::STRICTNE => handlers::strict_ne(vm),
        inst::NOT => handlers::not(vm),
        inst::POP => handlers::pop(vm),
        inst::RET => handlers::ret(vm),
        inst::LDGLOBAL => handlers::ldglobal(vm),
        inst::STOREGLOBAL => handlers::storeglobal(vm),
        inst::CALL => handlers::call(vm),
        inst::JMPFALSEP => handlers::jmpfalsep(vm),
        inst::JMP => handlers::jmp(vm),
        inst::STORELOCAL => handlers::storelocal(vm),
        inst::LDLOCAL => handlers::ldlocal(vm),
        inst::ARRAYLIT => handlers::arraylit(vm),
        inst::OBJLIT => handlers::objlit(vm),
        inst::STATICPROPACCESS => handlers::staticpropertyaccess(vm),
        inst::STATICPROPSET => handlers::staticpropertyset(vm),
        inst::STATICPROPSETW => handlers::staticpropertysetw(vm),
        inst::DYNAMICPROPSET => handlers::dynamicpropertyset(vm),
        inst::DYNAMICPROPACCESS => handlers::dynamicpropertyaccess(vm),
        inst::LDLOCALEXT => handlers::ldlocalext(vm),
        inst::STORELOCALEXT => handlers::storelocalext(vm),
        inst::TRY => handlers::try_block(vm),
        inst::TRYEND => handlers::try_end(vm),
        inst::THROW => handlers::throw(vm),
        inst::TYPEOF => handlers::type_of(vm),
        inst::YIELD => handlers::yield_(vm),
        inst::JMPFALSENP => handlers::jmpfalsenp(vm),
        inst::JMPTRUEP => handlers::jmptruep(vm),
        inst::JMPTRUENP => handlers::jmptruenp(vm),
        inst::JMPNULLISHP => handlers::jmpnullishp(vm),
        inst::JMPNULLISHNP => handlers::jmpnullishnp(vm),
        inst::IMPORTDYN => handlers::import_dyn(vm),
        inst::IMPORTSTATIC => handlers::import_static(vm),
        inst::EXPORTDEFAULT => handlers::export_default(vm),
        inst::EXPORTNAMED => handlers::export_named(vm),
        inst::THIS => handlers::this(vm),
        inst::GLOBAL => handlers::global_this(vm),
        inst::SUPER => handlers::super_(vm),
        inst::DEBUGGER => handlers::debugger(vm),
        inst::REVSTCK => handlers::revstck(vm),
        inst::NEG => handlers::neg(vm),
        inst::POS => handlers::pos(vm),
        inst::UNDEF => handlers::undef(vm),
        inst::AWAIT => handlers::await_(vm),
        inst::NAN => handlers::nan(vm),
        inst::INFINITY => handlers::infinity(vm),
        _ => unimplemented!("{}", instruction),
    }
}
