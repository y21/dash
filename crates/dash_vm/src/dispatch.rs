use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
    vec::Drain,
};

use crate::{frame::Frame, gc::handle::Handle, local::LocalScope, value::object::Object};

use super::{value::Value, Vm};
use dash_middle::compiler::{constant::Constant, instruction::Instruction};

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

pub struct DispatchContext<'a> {
    vm: &'a mut Vm,
}

impl<'a> DispatchContext<'a> {
    pub fn new(vm: &'a mut Vm) -> Self {
        Self { vm }
    }

    pub fn scope(&mut self) -> LocalScope<'_> {
        LocalScope::new(self)
    }

    pub fn get_local(&mut self, index: usize) -> Value {
        self.vm
            .get_local(index)
            .expect("Bytecode attempted to reference invalid local")
    }

    pub fn get_external(&mut self, index: usize) -> &Handle<dyn Object> {
        self.vm
            .get_external(index)
            .expect("Bytecode attempted to reference invalid external")
    }

    pub fn pop_frame(&mut self) -> Frame {
        self.frames
            .pop()
            .expect("Bytecode attempted to pop frame, but no frames exist")
    }

    pub fn pop_stack(&mut self) -> Value {
        self.stack
            .pop()
            .expect("Bytecode attempted to pop stack value, but nothing was on the stack")
    }

    pub fn peek_stack(&mut self) -> Value {
        self.stack
            .last()
            .expect("Bytecode attempted to peek stack value, but nothing was on the stack")
            .clone()
    }

    pub fn pop_stack2(&mut self) -> (Value, Value) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        (a, b)
    }

    pub fn pop_stack3(&mut self) -> (Value, Value, Value) {
        let c = self.stack.pop().unwrap();
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        (a, b, c)
    }

    pub fn pop_stack_many(&mut self, count: usize) -> Drain<Value> {
        let pos = self.stack.len() - count;
        self.stack.drain(pos..)
    }

    pub fn evaluate_binary_with_scope<F>(&mut self, fun: F) -> Result<Option<HandleResult>, Value>
    where
        F: Fn(&Value, &Value, &mut LocalScope) -> Result<Value, Value>,
    {
        let (left, right) = self.pop_stack2();
        let mut scope = self.scope();

        scope.add_value(left.clone());
        scope.add_value(right.clone());
        let result = fun(&left, &right, &mut scope)?;
        scope.try_push_stack(result)?;
        Ok(None)
    }

    pub fn active_frame(&self) -> &Frame {
        self.frames
            .last()
            .expect("Dispatch Context attempted to reference missing frame")
    }

    pub fn active_frame_mut(&mut self) -> &mut Frame {
        self.frames
            .last_mut()
            .expect("Dispatch Context attempted to reference missing frame")
    }

    pub fn constant(&self, index: usize) -> Constant {
        self.active_frame().function.constants[index].clone()
    }

    pub fn identifier_constant(&self, index: usize) -> Rc<str> {
        self.constant(index)
            .as_identifier()
            .cloned()
            .expect("Bytecode attempted to reference invalid identifier constant")
    }

    pub fn string_constant(&self, index: usize) -> Rc<str> {
        self.constant(index)
            .as_string()
            .cloned()
            .expect("Bytecode attempted to reference invalid string constant")
    }

    pub fn number_constant(&self, index: usize) -> f64 {
        self.constant(index)
            .as_number()
            .expect("Bytecode attempted to reference invalid number constant")
    }
}

impl<'a> Deref for DispatchContext<'a> {
    type Target = Vm;
    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}

impl<'a> DerefMut for DispatchContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vm
    }
}

mod handlers {
    use dash_middle::compiler::instruction::IntrinsicOperation;
    use dash_middle::compiler::FunctionCallMetadata;
    use dash_middle::compiler::ObjectMemberKind;
    use dash_middle::compiler::StaticImportKind;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::ops::Add;
    use std::ops::Div;
    use std::ops::Mul;
    use std::ops::Rem;
    use std::ops::Sub;

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

    fn constant_instruction(mut cx: DispatchContext<'_>, idx: usize) -> Result<(), Value> {
        let constant = cx.constant(idx);

        #[cfg(feature = "jit")]
        cx.record_constant(idx as u16, &constant);

        let value = Value::from_constant(constant, &mut cx);
        cx.try_push_stack(value)?;
        Ok(())
    }

    pub fn constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip();
        constant_instruction(cx, id as usize)?;
        Ok(None)
    }

    pub fn constantw(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetchw_and_inc_ip();
        constant_instruction(cx, id as usize)?;
        Ok(None)
    }

    pub fn add(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::add)
    }

    pub fn sub(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::sub)
    }

    pub fn mul(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::mul)
    }

    pub fn div(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::div)
    }

    pub fn rem(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::rem)
    }

    pub fn pow(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::pow)
    }

    pub fn bitor(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::bitor)
    }

    pub fn bitxor(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::bitxor)
    }

    pub fn bitand(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::bitand)
    }

    pub fn bitshl(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::bitshl)
    }

    pub fn bitshr(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::bitshr)
    }

    pub fn bitushr(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(Value::bitushr)
    }

    pub fn bitnot(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        let mut sc = cx.scope();
        sc.add_value(value.clone());
        let result = value.bitnot(&mut sc)?;
        sc.try_push_stack(result)?;
        Ok(None)
    }

    pub fn objin(_cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        todo!()
    }

    pub fn instanceof(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let (source, target) = cx.pop_stack2();

        let mut sc = cx.scope();
        sc.add_value(target.clone());
        sc.add_value(source.clone());

        let is_instanceof = source.instanceof(&target, &mut sc).map(Value::Boolean)?;
        sc.try_push_stack(is_instanceof)?;
        Ok(None)
    }

    pub fn lt(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(ValueEquality::lt)
    }

    pub fn le(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(ValueEquality::le)
    }

    pub fn gt(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(ValueEquality::gt)
    }

    pub fn ge(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(ValueEquality::ge)
    }

    pub fn eq(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(ValueEquality::eq)
    }

    pub fn ne(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(ValueEquality::ne)
    }

    pub fn strict_eq(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(ValueEquality::strict_eq)
    }

    pub fn strict_ne(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.evaluate_binary_with_scope(ValueEquality::strict_ne)
    }

    pub fn neg(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        let mut scope = cx.scope();
        let result = value.to_number(&mut scope)?;
        scope.try_push_stack(Value::number(-result))?;
        Ok(None)
    }

    pub fn pos(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        let mut scope = cx.scope();
        let result = value.to_number(&mut scope)?;
        scope.try_push_stack(Value::number(result))?;
        Ok(None)
    }

    pub fn not(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        let result = value.not();
        cx.try_push_stack(result)?;
        Ok(None)
    }

    pub fn pop(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.pop_stack();
        Ok(None)
    }

    pub fn ret(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let tc_depth = cx.fetchw_and_inc_ip();
        let value = cx.pop_stack();
        let this = cx.pop_frame();

        // Drain all try catch blocks that are in this frame.
        let lower_tcp = cx.try_blocks.len() - usize::from(tc_depth);
        drop(cx.try_blocks.drain(lower_tcp..));

        // Drain all the stack space from this frame
        drop(cx.stack.drain(this.sp..));

        match this.state {
            FrameState::Module(_) => {
                // Put it back on the frame stack, because we'll need it in Vm::execute_module
                cx.frames.push(this)
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

    pub fn ldglobal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip();
        let name = cx.identifier_constant(id.into());
        let mut scope = cx.scope();

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

    pub fn storeglobal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip();
        let name = cx.identifier_constant(id.into());
        let value = cx.pop_stack();

        let mut scope = cx.scope();
        scope.global.clone().set_property(
            &mut scope,
            ToString::to_string(&name).into(),
            PropertyValue::static_default(value.clone()),
        )?;
        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn call(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let meta = FunctionCallMetadata::from(cx.fetch_and_inc_ip());
        let argc = meta.value();
        let is_constructor = meta.is_constructor_call();
        let has_this = meta.is_object_call();

        let (args, refs) = {
            let argc = argc.into();
            let mut args = Vec::with_capacity(argc);
            let mut refs = Vec::new();

            let iter = cx.pop_stack_many(argc);

            for value in iter {
                if let Value::Object(handle) = &value {
                    refs.push(handle.clone());
                }

                args.push(value);
            }

            (args, refs)
        };

        let callee = cx.pop_stack();

        let this = if has_this { cx.pop_stack() } else { Value::undefined() };

        let mut scope = cx.scope();
        let scope_ref = &scope as *const LocalScope;
        scope.externals.add(scope_ref, refs);

        let ret = if is_constructor {
            callee.construct(&mut scope, this, args)?
        } else {
            callee.apply(&mut scope, this, args)?
        };

        scope.try_push_stack(ret)?;
        Ok(None)
    }

    pub fn jmpfalsep(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack();

        let jump = !value.is_truthy();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(jump);

        if jump {
            let frame = cx.active_frame_mut();

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmpfalsenp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = !value.is_truthy();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(jump);

        if jump {
            let frame = cx.active_frame_mut();

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmptruep(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack();

        let jump = value.is_truthy();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(jump);

        if jump {
            let frame = cx.active_frame_mut();

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmptruenp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = value.is_truthy();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(jump);

        if jump {
            let frame = cx.active_frame_mut();

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmpnullishp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack();

        let jump = value.is_nullish();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(jump);

        if jump {
            let frame = cx.active_frame_mut();

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmpnullishnp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = value.is_nullish();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(jump);

        if jump {
            let frame = cx.active_frame_mut();

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmpundefinedp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack();

        let jump = match value {
            Value::Undefined(..) => true,
            Value::Object(obj) | Value::External(obj) => {
                obj.as_primitive_capable().map(|p| p.is_undefined()).unwrap_or_default()
            }
            _ => false,
        };

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(jump);

        if jump {
            let frame = cx.active_frame_mut();

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmpundefinednp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = match value {
            Value::Undefined(..) => true,
            Value::Object(obj) | Value::External(obj) => {
                obj.as_primitive_capable().map(|p| p.is_undefined()).unwrap_or_default()
            }
            _ => false,
        };

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(jump);

        if jump {
            let frame = cx.active_frame_mut();

            if offset.is_negative() {
                frame.ip -= -offset as usize;
            } else {
                frame.ip += offset as usize;
            }
        }

        Ok(None)
    }

    pub fn jmp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let frame = cx.active_frame_mut();

        // Note: this is an unconditional jump, so we don't push this into the trace as a conditional jump

        if offset.is_negative() {
            #[cfg(feature = "jit")]
            let old_ip = frame.ip;

            frame.ip -= -offset as usize;

            // Negative jumps are (currently) always also a marker for the end of a loop
            // and we want to JIT compile loops that run often
            #[cfg(feature = "jit")]
            crate::jit::handle_loop_end(&mut cx, old_ip);
        } else {
            frame.ip += offset as usize;
        }

        Ok(None)
    }

    pub fn storelocal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip() as usize;
        let value = cx.pop_stack();

        #[cfg(feature = "jit")]
        cx.record_local(id as u16, &value);

        cx.set_local(id, value.clone());
        cx.try_push_stack(value)?;

        Ok(None)
    }

    pub fn ldlocal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip();
        let value = cx.get_local(id.into());

        #[cfg(feature = "jit")]
        cx.record_local(id as u16, &value);

        cx.try_push_stack(value)?;
        Ok(None)
    }

    pub fn arraylit(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let len = cx.fetch_and_inc_ip() as usize;

        let elements = cx
            .pop_stack_many(len)
            .map(PropertyValue::static_default)
            .collect::<Vec<_>>();
        let array = Array::from_vec(&mut cx, elements);
        let handle = cx.gc.register(array);
        cx.try_push_stack(Value::Object(handle))?;
        Ok(None)
    }

    pub fn objlit(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let len = cx.fetch_and_inc_ip() as usize;

        let mut obj = HashMap::new();
        for _ in 0..len {
            let kind = ObjectMemberKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

            let key = match kind {
                // TODO: it might be a symbol, don't to_string it then!
                ObjectMemberKind::Dynamic => {
                    // TODO: don't create LocalScope every time
                    match cx.pop_stack() {
                        Value::Symbol(sym) => PropertyKey::Symbol(sym),
                        value => {
                            let string = value.to_string(&mut cx.scope())?;
                            // TODO: can PropertyKey::String be a Rc<str>?
                            let string = Cow::Owned(String::from(&*string));
                            PropertyKey::String(string)
                        }
                    }
                }
                ObjectMemberKind::Getter | ObjectMemberKind::Setter | ObjectMemberKind::Static => {
                    let id = cx.fetch_and_inc_ip();

                    // TODO: optimization opportunity: do not reallocate string from Rc<str>
                    let key = String::from(cx.identifier_constant(id.into()).as_ref());
                    PropertyKey::String(Cow::Owned(key))
                }
            };
            let value = cx.pop_stack();

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

        let mut scope = cx.scope();
        let obj = NamedObject::with_values(&mut scope, obj);

        let handle = scope.gc.register(obj);
        scope.try_push_stack(handle.into())?;

        Ok(None)
    }

    pub fn staticpropertyaccess(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip();
        let ident = cx.identifier_constant(id.into());

        let preserve_this = cx.fetch_and_inc_ip() == 1;

        let target = if preserve_this { cx.peek_stack() } else { cx.pop_stack() };

        let mut scope = cx.scope();
        // TODO: add scope to externals because calling get_property can invoke getters

        let value = target.get_property(&mut scope, ident.as_ref().into())?;
        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn staticpropertyset(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip();
        let key = cx.identifier_constant(id.into());

        let (target, value) = cx.pop_stack2();

        let mut scope = cx.scope();
        target.set_property(
            &mut scope,
            ToString::to_string(&key).into(),
            PropertyValue::static_default(value.clone()),
        )?;

        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn staticpropertysetw(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetchw_and_inc_ip();
        let key = cx.identifier_constant(id.into());

        let (target, value) = cx.pop_stack2();

        let mut scope = cx.scope();
        target.set_property(
            &mut scope,
            ToString::to_string(&key).into(),
            PropertyValue::static_default(value.clone()),
        )?;

        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn dynamicpropertyset(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let (target, value, key) = cx.pop_stack3();
        let mut scope = cx.scope();

        let key = PropertyKey::from_value(&mut scope, key)?;
        target.set_property(&mut scope, key, PropertyValue::static_default(value.clone()))?;

        scope.try_push_stack(value)?;
        Ok(None)
    }

    pub fn dynamicpropertyaccess(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let key = cx.pop_stack();

        let preserve_this = cx.fetch_and_inc_ip() == 1;

        let mut scope = cx.scope();
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

    pub fn ldlocalext(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip();
        let value = Value::External(cx.get_external(id.into()).clone());

        // Unbox external values such that any use will create a copy
        let value = value.unbox_external();

        cx.try_push_stack(value)?;
        Ok(None)
    }

    pub fn storelocalext(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let id = cx.fetch_and_inc_ip();
        let value = cx.pop_stack();

        let external = cx.get_external(id.into()).as_ptr();
        // TODO: make sure that nothing really aliases this &mut
        unsafe { (*external).value = value.clone().into_boxed() };

        cx.try_push_stack(value)?;
        Ok(None)
    }

    pub fn try_block(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let ip = cx.active_frame().ip;
        let catch_offset = cx.fetchw_and_inc_ip() as usize;
        let catch_ip = ip + catch_offset + 2;
        let frame_ip = cx.frames.len();

        cx.try_blocks.push(TryBlock { catch_ip, frame_ip });

        Ok(None)
    }

    pub fn try_end(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.try_blocks.pop();
        Ok(None)
    }

    pub fn throw(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        Err(cx.pop_stack())
    }

    pub fn type_of(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        cx.try_push_stack(value.type_of().as_value())?;
        Ok(None)
    }

    pub fn yield_(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        Ok(Some(HandleResult::Yield(value)))
    }

    pub fn await_(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        Ok(Some(HandleResult::Await(value)))
    }

    pub fn import_dyn(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();

        let _ret = match cx.params.dynamic_import_callback() {
            Some(cb) => cb(&mut cx, value)?,
            None => throw!(cx, "Dynamic imports are disabled for this context"),
        };

        // TODO: dynamic imports are currently statements, making them useless
        // TODO: make them an expression and push ret on stack

        Ok(None)
    }

    pub fn import_static(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let ty = StaticImportKind::from_repr(cx.fetch_and_inc_ip()).expect("Invalid import kind");
        let local_id = cx.fetchw_and_inc_ip();
        let path_id = cx.fetchw_and_inc_ip();

        let path = cx.string_constant(path_id.into());

        let value = match cx.params.static_import_callback() {
            Some(cb) => cb(&mut cx, ty, &path)?,
            None => throw!(cx, "Static imports are disabled for this context."),
        };

        cx.set_local(local_id.into(), value);

        Ok(None)
    }

    pub fn export_default(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        let frame = cx.active_frame_mut();

        match &mut frame.state {
            FrameState::Module(module) => {
                module.default = Some(value);
            }
            _ => throw!(cx, "Export is only available at the top level in modules"),
        }

        Ok(None)
    }

    pub fn export_named(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        // let mut sc = cx.scope();
        let count = cx.fetchw_and_inc_ip();

        for _ in 0..count {
            let (value, ident) = match cx.fetch_and_inc_ip() {
                0 => {
                    // Local variable
                    let loc_id = cx.fetchw_and_inc_ip();
                    let ident_id = cx.fetchw_and_inc_ip();

                    let value = cx.get_local(loc_id.into());
                    let ident = cx.identifier_constant(ident_id.into());

                    (value, ident)
                }
                1 => {
                    // Global variable
                    let ident_id = cx.fetchw_and_inc_ip();

                    let ident = cx.identifier_constant(ident_id.into());

                    let global = cx.global.clone();
                    let value = global.get_property(&mut cx.scope(), ident.as_ref().into())?;

                    (value, ident)
                }
                _ => unreachable!(),
            };

            let frame = cx.active_frame_mut();
            match &mut frame.state {
                FrameState::Module(exports) => exports.named.push((ident, value)),
                _ => throw!(cx, "Export is only available at the top level in modules"),
            }
        }

        Ok(None)
    }

    pub fn debugger(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        if let Some(cb) = cx.params().debugger_callback() {
            cb(&mut cx)?;
        }

        Ok(None)
    }

    pub fn this(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let this = cx
            .frames
            .iter()
            .rev()
            .find_map(|f| f.this.as_ref())
            .cloned()
            .unwrap_or_else(|| Value::Object(cx.global.clone()));

        cx.try_push_stack(this)?;
        Ok(None)
    }

    pub fn global_this(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let global = cx.global.clone();
        cx.try_push_stack(Value::Object(global))?;
        Ok(None)
    }

    pub fn super_(cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        throw!(cx, "`super` keyword unexpected in this context");
    }

    pub fn undef(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.try_push_stack(Value::undefined())?;
        Ok(None)
    }

    pub fn infinity(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.try_push_stack(Value::number(f64::INFINITY))?;
        Ok(None)
    }

    pub fn nan(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        cx.try_push_stack(Value::number(f64::NAN))?;
        Ok(None)
    }

    pub fn call_symbol_iterator(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let value = cx.pop_stack();
        let mut scope = cx.scope();
        let symbol_iterator = scope.statics.symbol_iterator.clone();
        let iterable = value.get_property(&mut scope, PropertyKey::Symbol(symbol_iterator))?;
        let iterator = iterable.apply(&mut scope, value, Vec::new())?;
        scope.try_push_stack(iterator)?;
        Ok(None)
    }

    pub fn delete_property_dynamic(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let (property, target) = cx.pop_stack2();
        let mut scope = cx.scope();
        let key = PropertyKey::from_value(&mut scope, property)?;
        let value = target.delete_property(&mut scope, key)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value, Value::Undefined(..));
        scope.try_push_stack(Value::Boolean(did_delete))?;
        Ok(None)
    }

    pub fn delete_property_static(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let target = cx.pop_stack();
        let cid = cx.fetchw_and_inc_ip();
        let con = cx.identifier_constant(cid.into());
        let mut scope = cx.scope();
        let key = PropertyKey::from(con.as_ref());
        let value = target.delete_property(&mut scope, key)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value, Value::Undefined(..));
        scope.try_push_stack(Value::Boolean(did_delete))?;
        Ok(None)
    }

    pub fn switch(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let case_count = cx.fetchw_and_inc_ip();
        let has_default = cx.fetch_and_inc_ip() == 1;

        let switch_expr = cx.pop_stack();

        let mut target_ip = None;

        let mut scope = LocalScope::new(&mut cx);
        for _ in 0..case_count {
            let mut cx = DispatchContext::new(&mut scope);
            let case_value = cx.pop_stack();
            let case_offset = cx.fetchw_and_inc_ip() as usize;
            let ip = cx.active_frame().ip;
            drop(cx);

            let is_eq = switch_expr.strict_eq(&case_value, &mut scope)?.to_boolean()?;
            let has_matching_case = target_ip.is_some();

            if is_eq && !has_matching_case {
                target_ip = Some(ip + case_offset);
            }
        }

        let mut cx = DispatchContext::new(&mut scope);
        if has_default {
            let default_offset = cx.fetchw_and_inc_ip() as usize;
            let ip = cx.active_frame().ip;

            if target_ip.is_none() {
                target_ip = Some(ip + default_offset);
            }
        }

        if let Some(target_ip) = target_ip {
            cx.active_frame_mut().ip = target_ip;
        }

        Ok(None)
    }

    pub fn objdestruct(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let count = cx.fetchw_and_inc_ip();
        let obj = cx.pop_stack();
        let mut scope = cx.scope();

        for _ in 0..count {
            let mut cx = DispatchContext::new(&mut scope);
            let loc_id = cx.fetchw_and_inc_ip();
            let ident_id = cx.fetchw_and_inc_ip();

            let id = cx.number_constant(loc_id.into()) as usize;
            let ident = cx.identifier_constant(ident_id.into());
            drop(cx);

            let prop = obj.get_property(&mut scope, PropertyKey::from(ident.as_ref()))?;
            scope.set_local(id, prop);
        }

        Ok(None)
    }

    pub fn arraydestruct(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let count = cx.fetchw_and_inc_ip();
        let array = cx.pop_stack();
        let mut scope = cx.scope();

        for i in 0..count {
            let mut cx = DispatchContext::new(&mut scope);
            let loc_id = cx.fetchw_and_inc_ip();

            let id = cx.number_constant(loc_id.into()) as usize;
            drop(cx);

            let prop = array.get_property(&mut scope, PropertyKey::from(i.to_string().as_ref()))?;
            scope.set_local(id, prop);
        }

        Ok(None)
    }

    pub fn intrinsic_op(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Value> {
        let op = IntrinsicOperation::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! lr_as_num_spec {
            () => {{
                let (left, right) = cx.pop_stack2();
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => (l.0, r.0),
                    _ => unreachable!(),
                }
            }};
        }

        macro_rules! bin_op {
            ($fun:expr) => {{
                let (l, r) = lr_as_num_spec!();
                // No try_push_stack needed, because we just popped two values off. Therefore it can hold one more now
                cx.stack.push(Value::number($fun(l, r)));
            }};
        }

        macro_rules! bin_op_i64 {
            ($op:tt) => {{
                let (l, r) = lr_as_num_spec!();
                cx.stack.push(Value::number(((l as i32) $op (r as i32)) as f64));
            }};
        }

        macro_rules! bin_op_to_f64 {
            ($op:tt) => {{
                let (l, r) = lr_as_num_spec!();
                cx.stack.push(Value::number((l $op r) as i64 as f64));
            }};
        }

        macro_rules! postfix {
            ($op:tt) => {{
                let id = cx.fetch_and_inc_ip();
                let local = match cx.get_local(id.into()) {
                    Value::Number(n) => n,
                    _ => unreachable!(),
                };
                cx.set_local(id.into(), Value::number(local.0 $op 1.0));
                cx.try_push_stack(Value::Number(local))?;
            }};
        }

        macro_rules! prefix {
            ($op:tt) => {{{
                let id = cx.fetch_and_inc_ip();
                let local = match cx.get_local(id.into()) {
                    Value::Number(n) => n,
                    _ => unreachable!(),
                };
                let new = Value::number(local.0 $op 1.0);
                cx.set_local(id.into(), new.clone());
                cx.try_push_stack(new)?;
            }
            }};
        }

        macro_rules! bin_op_numl_constr {
            ($op:tt) => {{
                let left = match cx.pop_stack() {
                    Value::Number(n) => n.0,
                    _ => unreachable!(),
                };
                let right = cx.fetch_and_inc_ip() as f64;
                cx.try_push_stack(Value::Boolean(left $op right))?;
            }};
        }

        macro_rules! bin_op_numl_constr_n {
            ($op:tt, $ty:ty) => {{
                let left = match cx.pop_stack() {
                    Value::Number(n) => n.0,
                    _ => unreachable!(),
                };
                let mut right_bytes: [u8; <$ty>::BITS as usize / 8] = [0; <$ty>::BITS as usize / 8];
                for byte in right_bytes.iter_mut() {
                    *byte = cx.fetch_and_inc_ip();
                }
                let right = <$ty>::from_ne_bytes(right_bytes) as f64;
                cx.try_push_stack(Value::Boolean(left $op right))?;
            }};
        }

        match op {
            IntrinsicOperation::AddNumLR => bin_op!(Add::add),
            IntrinsicOperation::SubNumLR => bin_op!(Sub::sub),
            IntrinsicOperation::MulNumLR => bin_op!(Mul::mul),
            IntrinsicOperation::DivNumLR => bin_op!(Div::div),
            IntrinsicOperation::RemNumLR => bin_op!(Rem::rem),
            IntrinsicOperation::PowNumLR => bin_op!(f64::powf),
            IntrinsicOperation::GtNumLR => bin_op_to_f64!(>),
            IntrinsicOperation::GeNumLR => bin_op_to_f64!(>=),
            IntrinsicOperation::LtNumLR => bin_op_to_f64!(<),
            IntrinsicOperation::LeNumLR => bin_op_to_f64!(<=),
            IntrinsicOperation::EqNumLR => bin_op_to_f64!(==),
            IntrinsicOperation::NeNumLR => bin_op_to_f64!(!=),
            IntrinsicOperation::BitOrNumLR => bin_op_i64!(|),
            IntrinsicOperation::BitXorNumLR => bin_op_i64!(^),
            IntrinsicOperation::BitAndNumLR => bin_op_i64!(&),
            IntrinsicOperation::BitShlNumLR => bin_op_i64!(<<),
            IntrinsicOperation::BitShrNumLR => bin_op_i64!(>>),
            IntrinsicOperation::BitUshrNumLR => bin_op_i64!(>>),
            IntrinsicOperation::PostfixIncLocalNum => postfix!(+),
            IntrinsicOperation::PostfixDecLocalNum => postfix!(-),
            IntrinsicOperation::PrefixIncLocalNum => prefix!(+),
            IntrinsicOperation::PrefixDecLocalNum => prefix!(-),
            IntrinsicOperation::GtNumLConstR => bin_op_numl_constr!(>),
            IntrinsicOperation::GeNumLConstR => bin_op_numl_constr!(>=),
            IntrinsicOperation::LtNumLConstR => bin_op_numl_constr!(<),
            IntrinsicOperation::LeNumLConstR => bin_op_numl_constr!(<=),
            IntrinsicOperation::GtNumLConstR32 => bin_op_numl_constr_n!(>, u32),
            IntrinsicOperation::GeNumLConstR32 => bin_op_numl_constr_n!(>=, u32),
            IntrinsicOperation::LtNumLConstR32 => bin_op_numl_constr_n!(<, u32),
            IntrinsicOperation::LeNumLConstR32 => bin_op_numl_constr_n!(<=, u32),
        }

        Ok(None)
    }
}

pub fn handle(vm: &mut Vm, instruction: Instruction) -> Result<Option<HandleResult>, Value> {
    let cx = DispatchContext::new(vm);
    match instruction {
        Instruction::Constant => handlers::constant(cx),
        Instruction::ConstantW => handlers::constantw(cx),
        Instruction::Add => handlers::add(cx),
        Instruction::Sub => handlers::sub(cx),
        Instruction::Mul => handlers::mul(cx),
        Instruction::Div => handlers::div(cx),
        Instruction::Rem => handlers::rem(cx),
        Instruction::Pow => handlers::pow(cx),
        Instruction::BitOr => handlers::bitor(cx),
        Instruction::BitXor => handlers::bitxor(cx),
        Instruction::BitAnd => handlers::bitand(cx),
        Instruction::BitShl => handlers::bitshl(cx),
        Instruction::BitShr => handlers::bitshr(cx),
        Instruction::BitUshr => handlers::bitushr(cx),
        Instruction::BitNot => handlers::bitnot(cx),
        Instruction::ObjIn => handlers::objin(cx),
        Instruction::InstanceOf => handlers::instanceof(cx),
        Instruction::Gt => handlers::gt(cx),
        Instruction::Ge => handlers::ge(cx),
        Instruction::Lt => handlers::lt(cx),
        Instruction::Le => handlers::le(cx),
        Instruction::Eq => handlers::eq(cx),
        Instruction::Ne => handlers::ne(cx),
        Instruction::StrictEq => handlers::strict_eq(cx),
        Instruction::StrictNe => handlers::strict_ne(cx),
        Instruction::Not => handlers::not(cx),
        Instruction::Pop => handlers::pop(cx),
        Instruction::Ret => handlers::ret(cx),
        Instruction::LdGlobal => handlers::ldglobal(cx),
        Instruction::StoreGlobal => handlers::storeglobal(cx),
        Instruction::Call => handlers::call(cx),
        Instruction::JmpFalseP => handlers::jmpfalsep(cx),
        Instruction::Jmp => handlers::jmp(cx),
        Instruction::StoreLocal => handlers::storelocal(cx),
        Instruction::LdLocal => handlers::ldlocal(cx),
        Instruction::ArrayLit => handlers::arraylit(cx),
        Instruction::ObjLit => handlers::objlit(cx),
        Instruction::StaticPropAccess => handlers::staticpropertyaccess(cx),
        Instruction::StaticPropSet => handlers::staticpropertyset(cx),
        Instruction::StaticPropSetW => handlers::staticpropertysetw(cx),
        Instruction::DynamicPropSet => handlers::dynamicpropertyset(cx),
        Instruction::DynamicPropAccess => handlers::dynamicpropertyaccess(cx),
        Instruction::LdLocalExt => handlers::ldlocalext(cx),
        Instruction::StoreLocalExt => handlers::storelocalext(cx),
        Instruction::Try => handlers::try_block(cx),
        Instruction::TryEnd => handlers::try_end(cx),
        Instruction::Throw => handlers::throw(cx),
        Instruction::TypeOf => handlers::type_of(cx),
        Instruction::Yield => handlers::yield_(cx),
        Instruction::JmpFalseNP => handlers::jmpfalsenp(cx),
        Instruction::JmpTrueP => handlers::jmptruep(cx),
        Instruction::JmpTrueNP => handlers::jmptruenp(cx),
        Instruction::JmpNullishP => handlers::jmpnullishp(cx),
        Instruction::JmpNullishNP => handlers::jmpnullishnp(cx),
        Instruction::JmpUndefinedP => handlers::jmpundefinedp(cx),
        Instruction::JmpUndefinedNP => handlers::jmpundefinednp(cx),
        Instruction::ImportDyn => handlers::import_dyn(cx),
        Instruction::ImportStatic => handlers::import_static(cx),
        Instruction::ExportDefault => handlers::export_default(cx),
        Instruction::ExportNamed => handlers::export_named(cx),
        Instruction::This => handlers::this(cx),
        Instruction::Global => handlers::global_this(cx),
        Instruction::Super => handlers::super_(cx),
        Instruction::Debugger => handlers::debugger(cx),
        Instruction::Neg => handlers::neg(cx),
        Instruction::Pos => handlers::pos(cx),
        Instruction::Undef => handlers::undef(cx),
        Instruction::Await => handlers::await_(cx),
        Instruction::Nan => handlers::nan(cx),
        Instruction::Infinity => handlers::infinity(cx),
        Instruction::CallSymbolIterator => handlers::call_symbol_iterator(cx),
        Instruction::DeletePropertyDynamic => handlers::delete_property_dynamic(cx),
        Instruction::DeletePropertyStatic => handlers::delete_property_static(cx),
        Instruction::Switch => handlers::switch(cx),
        Instruction::ObjDestruct => handlers::objdestruct(cx),
        Instruction::ArrayDestruct => handlers::arraydestruct(cx),
        Instruction::Nop => Ok(None),
        Instruction::IntrinsicOp => handlers::intrinsic_op(cx),
        _ => unimplemented!("{:?}", instruction),
    }
}
