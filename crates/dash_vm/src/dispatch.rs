#![allow(clippy::needless_lifetimes)] // for now

use dash_log::warn;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
    vec::Drain,
};

use crate::{
    frame::Frame,
    gc::handle::Handle,
    localscope::LocalScope,
    value::{ExternalValue, Unrooted},
};

use super::{value::Value, Vm};
use dash_middle::compiler::{constant::Constant, instruction::Instruction};

// TODO: all of these should be `Unrooted`
pub enum HandleResult {
    Return(Unrooted),
    Yield(Unrooted),
    Await(Unrooted),
}

impl HandleResult {
    pub fn into_value(self) -> Unrooted {
        match self {
            HandleResult::Return(v) => v,
            HandleResult::Yield(v) => v,
            HandleResult::Await(v) => v,
        }
    }

    pub fn into_rooted(self, sc: &mut LocalScope) -> Value {
        match self {
            HandleResult::Return(v) => v,
            HandleResult::Yield(v) => v,
            HandleResult::Await(v) => v,
        }
        .root(sc)
    }
}

pub struct DispatchContext<'sc, 'vm> {
    scope: &'sc mut LocalScope<'vm>,
}

impl<'sc, 'vm> DispatchContext<'sc, 'vm> {
    pub fn new(scope: &'sc mut LocalScope<'vm>) -> Self {
        Self { scope }
    }

    pub fn get_local(&mut self, index: usize) -> Value {
        self.scope
            .get_local(index)
            .expect("Bytecode attempted to reference invalid local")
    }

    pub fn get_external(&mut self, index: usize) -> &Handle<ExternalValue> {
        self.scope
            .get_external(index)
            .expect("Bytecode attempted to reference invalid external")
    }

    pub fn pop_frame(&mut self) -> Frame {
        self.frames
            .pop()
            .expect("Bytecode attempted to pop frame, but no frames exist")
    }

    pub fn pop_stack(&mut self) -> Unrooted {
        self.scope.pop_stack_unwrap()
    }

    pub fn pop_stack_rooted(&mut self) -> Value {
        self.scope.pop_stack_unwrap().root(self.scope)
    }

    pub fn peek_stack(&mut self) -> Value {
        self.stack
            .last()
            .expect("Bytecode attempted to peek stack value, but nothing was on the stack")
            .clone()
    }

    // TODO: !! should return [Unrooted; N] !!
    fn pop_stack_const<const N: usize>(&mut self) -> [Value; N] {
        assert!(self.stack.len() >= N);
        // SAFETY: n pops are safe because we've checked the length
        // Sadly unsafe is needed here, see https://github.com/rust-lang/rust/issues/71257
        // TODO: remove this once the issue is fixed
        let mut arr: [Value; N] = std::array::from_fn(|_| unsafe { self.stack.pop().unwrap_unchecked() });
        arr.reverse();
        arr
    }

    pub fn pop_stack2_new(&mut self) -> (Unrooted, Unrooted) {
        let [a, b] = self.pop_stack_const().map(Unrooted::new);
        (a, b)
    }

    pub fn pop_stack2_rooted(&mut self) -> (Value, Value) {
        let [a, b] = self.pop_stack_const();
        self.scope.add_value(a.clone());
        self.scope.add_value(b.clone());
        (a, b)
    }

    pub fn pop_stack3(&mut self) -> (Value, Value, Value) {
        let [a, b, c] = self.pop_stack_const();
        (a, b, c)
    }

    pub fn pop_stack3_new(&mut self) -> (Unrooted, Unrooted, Unrooted) {
        let [a, b, c] = self.pop_stack_const().map(Unrooted::new);
        (a, b, c)
    }

    pub fn pop_stack3_rooted(&mut self) -> (Value, Value, Value) {
        let [a, b, c] = self.pop_stack_const();
        self.scope.add_value(a.clone());
        self.scope.add_value(b.clone());
        self.scope.add_value(c.clone());
        (a, b, c)
    }

    pub fn pop_stack_many(&mut self, count: usize) -> Drain<Value> {
        let pos = self.stack.len() - count;
        self.stack.drain(pos..)
    }

    pub fn evaluate_binary_with_scope<F>(&mut self, fun: F) -> Result<Option<HandleResult>, Unrooted>
    where
        F: Fn(&Value, &Value, &mut LocalScope) -> Result<Value, Value>,
    {
        let (left, right) = self.pop_stack2_new();

        let left = left.root(self.scope);
        let right = right.root(self.scope);

        let result = fun(&left, &right, self)?;
        self.stack.push(result);
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

impl<'sc, 'vm> Deref for DispatchContext<'sc, 'vm> {
    type Target = LocalScope<'vm>;
    fn deref(&self) -> &Self::Target {
        self.scope
    }
}

impl<'sc, 'vm> DerefMut for DispatchContext<'sc, 'vm> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.scope
    }
}

mod handlers {
    use dash_middle::compiler::instruction::AssignKind;
    use dash_middle::compiler::instruction::IntrinsicOperation;
    use dash_middle::compiler::ArrayMemberKind;
    use dash_middle::compiler::FunctionCallMetadata;
    use dash_middle::compiler::ObjectMemberKind;
    use dash_middle::compiler::StaticImportKind;
    use if_chain::if_chain;
    use smallvec::SmallVec;
    use std::borrow::Cow;
    use std::ops::Add;
    use std::ops::Div;
    use std::ops::Mul;
    use std::ops::Rem;
    use std::ops::Sub;

    use crate::frame::Frame;
    use crate::frame::FrameState;
    use crate::frame::TryBlock;
    use crate::localscope::LocalScope;
    use crate::throw;
    use crate::util::unlikely;
    use crate::value::array::Array;
    use crate::value::array::ArrayIterator;
    use crate::value::function::adjust_stack_from_flat_call;
    use crate::value::function::user::UserFunction;
    use crate::value::function::Function;
    use crate::value::function::FunctionKind;
    use crate::value::object::NamedObject;
    use crate::value::object::Object;
    use crate::value::object::ObjectMap;
    use crate::value::object::PropertyKey;
    use crate::value::object::PropertyValue;
    use crate::value::object::PropertyValueKind;
    use crate::value::ops::abstractions::conversions::ValueConversion;
    use crate::value::ops::equality::ValueEquality;

    use super::*;

    fn constant_instruction<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>, idx: usize) -> Result<(), Value> {
        let constant = cx.constant(idx);

        let value = Value::from_constant(constant, &mut cx);
        cx.stack.push(value);
        Ok(())
    }

    pub fn constant<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        constant_instruction(cx, id as usize)?;
        Ok(None)
    }

    pub fn constantw<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        constant_instruction(cx, id as usize)?;
        Ok(None)
    }

    pub fn add<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::add)
    }

    pub fn sub<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::sub)
    }

    pub fn mul<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::mul)
    }

    pub fn div<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::div)
    }

    pub fn rem<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::rem)
    }

    pub fn pow<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::pow)
    }

    pub fn bitor<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitor)
    }

    pub fn bitxor<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitxor)
    }

    pub fn bitand<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitand)
    }

    pub fn bitshl<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitshl)
    }

    pub fn bitshr<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitshr)
    }

    pub fn bitushr<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitushr)
    }

    pub fn bitnot<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.bitnot(&mut cx)?;
        cx.stack.push(result);
        Ok(None)
    }

    pub fn objin<'sc, 'vm>(cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        throw!(cx, Error, "in keyword is unimplemented");
    }

    pub fn instanceof<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let (source, target) = cx.pop_stack2_rooted();

        let is_instanceof = source.instanceof(&target, &mut cx).map(Value::Boolean)?;
        cx.stack.push(is_instanceof);
        Ok(None)
    }

    pub fn lt<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(ValueEquality::lt)
    }

    pub fn le<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(ValueEquality::le)
    }

    pub fn gt<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(ValueEquality::gt)
    }

    pub fn ge<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(ValueEquality::ge)
    }

    pub fn eq<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(ValueEquality::eq)
    }

    pub fn ne<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(ValueEquality::ne)
    }

    pub fn strict_eq<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(ValueEquality::strict_eq)
    }

    pub fn strict_ne<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(ValueEquality::strict_ne)
    }

    pub fn neg<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.to_number(&mut cx)?;
        cx.stack.push(Value::number(-result));
        Ok(None)
    }

    pub fn pos<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.to_number(&mut cx)?;
        cx.stack.push(Value::number(result));
        Ok(None)
    }

    pub fn not<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.not();
        cx.stack.push(result);
        Ok(None)
    }

    pub fn pop<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.pop_stack();
        Ok(None)
    }

    pub fn ret<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let tc_depth = cx.fetchw_and_inc_ip();
        let value = cx.pop_stack_rooted();
        let this = cx.pop_frame();

        // Drain all try catch blocks that are in this frame.
        let lower_tcp = cx.try_blocks.len() - usize::from(tc_depth);
        drop(cx.try_blocks.drain(lower_tcp..));

        // Drain all the stack space from this frame
        drop(cx.stack.drain(this.sp..));

        match this.state {
            FrameState::Module(_) => {
                // Put it back on the frame stack, because we'll need it in Vm::execute_module
                cx.frames.push(this);
                Ok(Some(HandleResult::Return(Unrooted::new(value))))
            }
            FrameState::Function {
                is_constructor_call,
                is_flat_call,
            } => {
                if_chain! {
                    if is_constructor_call && !matches!(value, Value::Object(_) | Value::External(_));
                    if let Frame { this: Some(this), .. } = this;
                    then {
                        // If this is a constructor call and the return value is not an object,
                        // return `this`
                        if is_flat_call {
                            cx.stack.push(this);
                            Ok(None)
                        } else {
                            Ok(Some(HandleResult::Return(Unrooted::new(this))))
                        }
                    }
                    else {
                        if is_flat_call {
                            cx.stack.push(value);
                            Ok(None)
                        } else {
                            Ok(Some(HandleResult::Return(Unrooted::new(value))))
                        }
                    }
                }
            }
        }
    }

    pub fn ldglobal<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let name = cx.identifier_constant(id.into());

        let value = match cx.global.as_any().downcast_ref::<NamedObject>() {
            Some(value) => match value.get_raw_property(name.as_ref().into()) {
                Some(value) => value.kind().get_or_apply(&mut cx, Value::undefined())?,
                None => throw!(&mut cx, ReferenceError, "{} is not defined", name),
            },
            None => cx.global.clone().get_property(&mut cx, name.as_ref().into())?,
        };

        cx.stack.push(value);
        Ok(None)
    }

    pub fn storeglobal<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let name = cx.identifier_constant(id.into());
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let right = cx.pop_stack_rooted();
                let value = cx
                    .global
                    .clone()
                    .get_property(&mut cx, PropertyKey::String(Cow::Borrowed(&name)))?;
                cx.scope.add_value(value.clone());

                let res = $op(&value, &right, &mut cx)?;
                cx.global.clone().set_property(
                    &mut cx,
                    ToString::to_string(&name).into(),
                    PropertyValue::static_default(res.clone()),
                )?;
                cx.stack.push(res);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let value = cx
                    .global
                    .clone()
                    .get_property(&mut cx, PropertyKey::String(Cow::Borrowed(&name)))?;
                let value = Value::number(value.to_number(&mut cx)?);

                let right = Value::number(1.0);
                let res = $op(&value, &right, &mut cx)?;
                cx.global.clone().set_property(
                    &mut cx,
                    ToString::to_string(&name).into(),
                    PropertyValue::static_default(res.clone()),
                )?;
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let value = cx
                    .global
                    .clone()
                    .get_property(&mut cx, PropertyKey::String(Cow::Borrowed(&name)))?;
                let value = Value::number(value.to_number(&mut cx)?);

                let right = Value::number(1.0);
                let res = $op(&value, &right, &mut cx)?;
                cx.global.clone().set_property(
                    &mut cx,
                    ToString::to_string(&name).into(),
                    PropertyValue::static_default(res),
                )?;
                cx.stack.push(value);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let value = cx.pop_stack_rooted();

                cx.global.clone().set_property(
                    &mut cx,
                    ToString::to_string(&name).into(),
                    PropertyValue::static_default(value.clone()),
                )?;
                cx.stack.push(value);
            }
            AssignKind::AddAssignment => op!(Value::add),
            AssignKind::SubAssignment => op!(Value::sub),
            AssignKind::MulAssignment => op!(Value::mul),
            AssignKind::DivAssignment => op!(Value::div),
            AssignKind::RemAssignment => op!(Value::rem),
            AssignKind::PowAssignment => op!(Value::pow),
            AssignKind::ShlAssignment => op!(Value::bitshl),
            AssignKind::ShrAssignment => op!(Value::bitshr),
            AssignKind::UshrAssignment => op!(Value::bitushr),
            AssignKind::BitAndAssignment => op!(Value::bitand),
            AssignKind::BitOrAssignment => op!(Value::bitor),
            AssignKind::BitXorAssignment => op!(Value::bitxor),
            AssignKind::PrefixIncrement => prefix!(Value::add),
            AssignKind::PostfixIncrement => postfix!(Value::add),
            AssignKind::PrefixDecrement => prefix!(Value::sub),
            AssignKind::PostfixDecrement => postfix!(Value::sub),
        }
        Ok(None)
    }

    /// Calls a function in a "non-recursive" way
    #[allow(clippy::too_many_arguments)]
    fn call_flat<'sc, 'vm>(
        mut cx: DispatchContext<'sc, 'vm>,
        callee: &Value,
        this: Value,
        function: &Function,
        user_function: &UserFunction,
        argc: usize,
        is_constructor: bool,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let sp = cx.stack.len() - argc;
        let Value::Object(callee) = callee else {
            unreachable!("guaranteed by caller")
        };

        let this = match is_constructor {
            true => Value::Object(function.new_instance(callee.clone(), &mut cx)?),
            false => this,
        };

        let len = cx.fetch_and_inc_ip();
        // If we have spread args, we need to "splice" values from iterables in.
        // This is hopefully rather "rare" (compared to regular call arguments),
        // so we can afford to do more work here in order to keep the common path fast.
        if len > 0 {
            let spread_indices: SmallVec<[_; 4]> = (0..len).map(|_| cx.fetch_and_inc_ip()).collect();
            let mut spread_count = 0;

            for spread_index in spread_indices {
                let adjusted_spread_index = sp + spread_count + spread_index as usize;

                let iterable = cx.stack[adjusted_spread_index].clone();
                let length = iterable.length_of_array_like(cx.scope)?;

                let mut splice_args = SmallVec::<[_; 4]>::new();

                for i in 0..length {
                    let value = iterable.get_property(&mut cx, i.to_string().into())?;
                    splice_args.push(value);
                }
                cx.stack
                    .splice(adjusted_spread_index..=adjusted_spread_index, splice_args);

                spread_count += length.saturating_sub(1);
            }
        }

        // NOTE: since we are in a "flat" call,
        // we don't need to add objects to the external
        // reference list since they stay on the VM stack
        // and are reachable from there

        adjust_stack_from_flat_call(&mut cx, user_function, sp, argc);

        let mut frame = Frame::from_function(Some(this), user_function, is_constructor, true);
        frame.set_sp(sp);

        cx.pad_stack_for_frame(&frame);
        cx.try_push_frame(frame)?;

        Ok(None)
    }

    /// Fallback for callable values that are not "function objects"
    fn call_generic<'sc, 'vm>(
        mut cx: DispatchContext<'sc, 'vm>,
        callee: &Value,
        this: Value,
        argc: usize,
        is_constructor: bool,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let (args, refs) = {
            let mut args = Vec::with_capacity(argc);
            let mut refs = Vec::new();

            let len = cx.fetch_and_inc_ip();
            let spread_indices: SmallVec<[_; 4]> = (0..len).map(|_| cx.fetch_and_inc_ip()).collect();

            let iter = cx.pop_stack_many(argc);

            if len == 0 {
                // Fast path for no spread arguments
                for value in iter {
                    if let Value::Object(handle) = &value {
                        refs.push(handle.clone());
                    }

                    args.push(value);
                }
            } else {
                let raw_args: SmallVec<[_; 4]> = iter.collect();
                let mut indices_iter = spread_indices.into_iter().peekable();

                for (index, value) in raw_args.into_iter().enumerate() {
                    if indices_iter.peek().is_some_and(|&v| usize::from(v) == index) {
                        let len = value.length_of_array_like(cx.scope)?;
                        for _ in 0..len {
                            let value = value.get_property(&mut cx, index.to_string().into())?;
                            if let Value::Object(handle) = &value {
                                refs.push(handle.clone());
                            }
                            args.push(value);
                        }
                        indices_iter.next();
                    } else {
                        if let Value::Object(handle) = &value {
                            refs.push(handle.clone());
                        }
                        args.push(value);
                    }
                }
            }

            (args, refs)
        };

        cx.scope.add_many(refs);

        let ret = if is_constructor {
            callee.construct(&mut cx, this, args)?
        } else {
            callee.apply(&mut cx, this, args)?
        };

        cx.stack.push(ret);
        Ok(None)
    }

    pub fn call<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let meta = FunctionCallMetadata::from(cx.fetch_and_inc_ip());
        let argc = usize::from(meta.value());
        let is_constructor = meta.is_constructor_call();
        let has_this = meta.is_object_call();

        let stack_len = cx.stack.len();
        let (callee, this) = if has_this {
            cx.stack[stack_len - argc - 2..].rotate_left(2);
            let (this, callee) = cx.pop_stack2_rooted();
            (callee, this)
        } else {
            cx.stack[stack_len - argc - 1..].rotate_left(1);
            // NOTE: Does not need to be rooted for flat calls. `generic_call` manually roots it.
            let callee = cx.pop_stack_rooted();
            (callee, Value::undefined())
        };

        if_chain! {
            if let Some(function) = callee.downcast_ref::<Function>();
            if let FunctionKind::User(user_function) = function.kind();
            then {
                call_flat(cx, &callee, this, function, user_function, argc, is_constructor)
            } else {
                call_generic(cx, &callee, this, argc, is_constructor)
            }
        }
    }

    pub fn jmpfalsep<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;

        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack_rooted();

        let jump = !value.is_truthy();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(ip, jump);

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

    pub fn jmpfalsenp<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = !value.is_truthy();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(ip, jump);

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

    pub fn jmptruep<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;

        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack_rooted();

        let jump = value.is_truthy();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(ip, jump);

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

    pub fn jmptruenp<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = value.is_truthy();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(ip, jump);

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

    pub fn jmpnullishp<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack_rooted();

        let jump = value.is_nullish();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(ip, jump);

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

    pub fn jmpnullishnp<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = value.is_nullish();

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(ip, jump);

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

    pub fn jmpundefinedp<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack_rooted();

        let jump = match value {
            Value::Undefined(..) => true,
            Value::Object(obj) => obj.as_primitive_capable().map(|p| p.is_undefined()).unwrap_or_default(),
            Value::External(obj) => obj.as_primitive_capable().map(|p| p.is_undefined()).unwrap_or_default(),
            _ => false,
        };

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(ip, jump);

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

    pub fn jmpundefinednp<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = match value {
            Value::Undefined(..) => true,
            Value::Object(obj) => obj.as_primitive_capable().map(|p| p.is_undefined()).unwrap_or_default(),
            Value::External(obj) => obj.as_primitive_capable().map(|p| p.is_undefined()).unwrap_or_default(),
            _ => false,
        };

        #[cfg(feature = "jit")]
        cx.record_conditional_jump(ip, jump);

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

    pub fn jmp<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
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

    pub fn storelocal<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip() as usize;
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let value = cx.get_local(id);
                let right = cx.pop_stack_rooted();
                let res = $op(&value, &right, &mut cx)?;
                cx.set_local(id, res.clone().into());
                cx.stack.push(res);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let value = cx.get_local(id);
                let value = Value::number(value.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(&value, &one, &mut cx)?;
                cx.set_local(id, res.clone().into());
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let value = cx.get_local(id);
                let value = Value::number(value.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(&value, &one, &mut cx)?;
                cx.set_local(id, res.into());
                cx.stack.push(value);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                // NOTE: Does not need to be rooted.
                let value = cx.pop_stack();
                cx.set_local(id, value.clone());
                cx.push_stack(value);
            }
            AssignKind::AddAssignment => op!(Value::add),
            AssignKind::SubAssignment => op!(Value::sub),
            AssignKind::MulAssignment => op!(Value::mul),
            AssignKind::DivAssignment => op!(Value::div),
            AssignKind::RemAssignment => op!(Value::rem),
            AssignKind::PowAssignment => op!(Value::pow),
            AssignKind::ShlAssignment => op!(Value::bitshl),
            AssignKind::ShrAssignment => op!(Value::bitshr),
            AssignKind::UshrAssignment => op!(Value::bitushr),
            AssignKind::BitAndAssignment => op!(Value::bitand),
            AssignKind::BitOrAssignment => op!(Value::bitor),
            AssignKind::BitXorAssignment => op!(Value::bitxor),
            AssignKind::PrefixIncrement => prefix!(Value::add),
            AssignKind::PostfixIncrement => postfix!(Value::add),
            AssignKind::PrefixDecrement => prefix!(Value::sub),
            AssignKind::PostfixDecrement => postfix!(Value::sub),
        }

        Ok(None)
    }

    pub fn ldlocal<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let value = cx.get_local(id.into()).unbox_external();

        cx.stack.push(value);
        Ok(None)
    }

    pub fn arraylit<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let len = cx.fetch_and_inc_ip() as usize;

        // No need to root/unroot anything here, GC cant trigger
        let elements = cx.pop_stack_many(len).collect::<Vec<_>>();
        let mut new_elements = Vec::with_capacity(elements.len());
        for value in elements {
            let id = ArrayMemberKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

            match id {
                ArrayMemberKind::Item => {
                    new_elements.push(PropertyValue::static_default(value));
                }
                ArrayMemberKind::Spread => {
                    // TODO: make this work for array-like values, not just arrays, by calling @@iterator on it
                    let len = value.length_of_array_like(cx.scope)?;
                    for i in 0..len {
                        let value = value.get_property(cx.scope, i.to_string().into())?;
                        new_elements.push(PropertyValue::static_default(value));
                    }
                }
            }
        }
        let array = Array::from_vec(&cx, new_elements);
        let handle = cx.gc.register(array);
        cx.stack.push(Value::Object(handle));
        Ok(None)
    }

    pub fn objlit<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let len = cx.fetch_and_inc_ip() as usize;

        let mut obj = ObjectMap::default();
        for _ in 0..len {
            let kind = ObjectMemberKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

            match kind {
                ObjectMemberKind::Dynamic => {
                    let key = cx.pop_stack_rooted();
                    let key = PropertyKey::from_value(cx.scope, key)?;
                    let value = cx.pop_stack_rooted();

                    obj.insert(key, PropertyValue::static_default(value));
                }
                ObjectMemberKind::Static => {
                    let id = cx.fetchw_and_inc_ip();
                    // TODO: optimization opportunity: do not reallocate string from Rc<str>
                    let key = String::from(cx.identifier_constant(id.into()).as_ref());
                    let value = cx.pop_stack_rooted();
                    obj.insert(
                        PropertyKey::String(Cow::Owned(key)),
                        PropertyValue::static_default(value),
                    );
                }
                ObjectMemberKind::Getter => {
                    let id = cx.fetchw_and_inc_ip();
                    let key = PropertyKey::String(Cow::Owned(String::from(cx.identifier_constant(id.into()).as_ref())));
                    let Value::Object(value) = cx.pop_stack_rooted() else {
                        panic!("Getter is not an object");
                    };
                    obj.entry(key)
                        .and_modify(|v| match v.kind_mut() {
                            PropertyValueKind::Trap { get, .. } => *get = Some(value.clone()),
                            _ => *v = PropertyValue::getter_default(value.clone()),
                        })
                        .or_insert_with(|| PropertyValue::getter_default(value.clone()));
                }
                ObjectMemberKind::Setter => {
                    let id = cx.fetchw_and_inc_ip();
                    let key = PropertyKey::String(Cow::Owned(String::from(cx.identifier_constant(id.into()).as_ref())));
                    let Value::Object(value) = cx.pop_stack_rooted() else {
                        panic!("Setter is not an object");
                    };
                    obj.entry(key)
                        .and_modify(|v| match v.kind_mut() {
                            PropertyValueKind::Trap { set, .. } => *set = Some(value.clone()),
                            _ => *v = PropertyValue::setter_default(value.clone()),
                        })
                        .or_insert_with(|| PropertyValue::setter_default(value.clone()));
                }
                ObjectMemberKind::Spread => {
                    let value = cx.pop_stack_rooted();
                    if let Value::Object(target) = &value {
                        for key in target.own_keys()? {
                            let key = PropertyKey::from_value(cx.scope, key)?;
                            let value = target.get_property(&mut cx, key.clone())?;
                            obj.insert(key, PropertyValue::static_default(value));
                        }
                    }
                }
            };
        }

        let obj = NamedObject::with_values(&cx, obj);

        let handle = cx.gc.register(obj);
        cx.stack.push(handle.into());

        Ok(None)
    }

    pub fn staticpropertyaccess<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let ident = cx.identifier_constant(id.into());

        let preserve_this = cx.fetch_and_inc_ip() == 1;

        let target = if preserve_this {
            cx.peek_stack()
        } else {
            cx.pop_stack_rooted()
        };

        let value = target.get_property(&mut cx, ident.as_ref().into())?;
        cx.stack.push(value);
        Ok(None)
    }

    pub fn staticpropertyassign<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();
        let id = cx.fetchw_and_inc_ip();
        let key = cx.identifier_constant(id.into());

        macro_rules! op {
            ($op:expr) => {{
                let (target, value) = cx.pop_stack2_new();

                let target = target.root(cx.scope);
                let value = value.root(cx.scope);

                let p = target.get_property(&mut cx, PropertyKey::String(Cow::Borrowed(&key)))?;
                let res = $op(&p, &value, &mut cx)?;

                target.set_property(
                    &mut cx,
                    ToString::to_string(&key).into(),
                    PropertyValue::static_default(res.clone()),
                )?;
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let target = cx.pop_stack_rooted();
                let prop = target.get_property(&mut cx, PropertyKey::String(Cow::Borrowed(&key)))?;
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(&prop, &one, &mut cx)?;
                target.set_property(
                    &mut cx,
                    ToString::to_string(&key).into(),
                    PropertyValue::static_default(res),
                )?;
                cx.stack.push(prop);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let target = cx.pop_stack_rooted();
                let prop = target.get_property(&mut cx, PropertyKey::String(Cow::Borrowed(&key)))?;
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(&prop, &one, &mut cx)?;
                target.set_property(
                    &mut cx,
                    ToString::to_string(&key).into(),
                    PropertyValue::static_default(res.clone()),
                )?;
                cx.stack.push(res);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let (target, value) = cx.pop_stack2_new();
                let target = target.root(cx.scope);
                let value = value.root(cx.scope);
                target.set_property(
                    &mut cx,
                    ToString::to_string(&key).into(),
                    PropertyValue::static_default(value.clone()),
                )?;
                cx.stack.push(value);
            }
            AssignKind::AddAssignment => op!(Value::add),
            AssignKind::SubAssignment => op!(Value::sub),
            AssignKind::MulAssignment => op!(Value::mul),
            AssignKind::DivAssignment => op!(Value::div),
            AssignKind::RemAssignment => op!(Value::rem),
            AssignKind::PowAssignment => op!(Value::pow),
            AssignKind::ShlAssignment => op!(Value::bitshl),
            AssignKind::ShrAssignment => op!(Value::bitshr),
            AssignKind::UshrAssignment => op!(Value::bitushr),
            AssignKind::BitAndAssignment => op!(Value::bitand),
            AssignKind::BitOrAssignment => op!(Value::bitor),
            AssignKind::BitXorAssignment => op!(Value::bitxor),
            AssignKind::PrefixIncrement => prefix!(Value::add),
            AssignKind::PostfixIncrement => postfix!(Value::add),
            AssignKind::PrefixDecrement => prefix!(Value::sub),
            AssignKind::PostfixDecrement => postfix!(Value::sub),
        };

        Ok(None)
    }

    pub fn dynamicpropertyassign<'sc, 'vm>(
        mut cx: DispatchContext<'sc, 'vm>,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let (target, value, key) = cx.pop_stack3_rooted();

                let key = PropertyKey::from_value(&mut cx, key)?;
                let prop = target.get_property(&mut cx, key.clone())?;

                let result = $op(&prop, &value, &mut cx)?;

                target.set_property(&mut cx, key, PropertyValue::static_default(result.clone()))?;
                cx.stack.push(result);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let (target, key) = cx.pop_stack2_rooted();
                let key = PropertyKey::from_value(&mut cx, key)?;
                let prop = target.get_property(&mut cx, key.clone())?;
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(&prop, &one, &mut cx)?;
                target.set_property(&mut cx, key, PropertyValue::static_default(res))?;
                cx.stack.push(prop);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let (target, key) = cx.pop_stack2_rooted();
                let key = PropertyKey::from_value(&mut cx, key)?;
                let prop = target.get_property(&mut cx, key.clone())?;
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(&prop, &one, &mut cx)?;
                target.set_property(&mut cx, key, PropertyValue::static_default(res.clone()))?;
                cx.stack.push(res);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let (target, value, key) = cx.pop_stack3_rooted();

                let key = PropertyKey::from_value(&mut cx, key)?;

                target.set_property(&mut cx, key, PropertyValue::static_default(value.clone()))?;
                cx.stack.push(value);
            }
            AssignKind::AddAssignment => op!(Value::add),
            AssignKind::SubAssignment => op!(Value::sub),
            AssignKind::MulAssignment => op!(Value::mul),
            AssignKind::DivAssignment => op!(Value::div),
            AssignKind::RemAssignment => op!(Value::rem),
            AssignKind::PowAssignment => op!(Value::pow),
            AssignKind::ShlAssignment => op!(Value::bitshl),
            AssignKind::ShrAssignment => op!(Value::bitshr),
            AssignKind::UshrAssignment => op!(Value::bitushr),
            AssignKind::BitAndAssignment => op!(Value::bitand),
            AssignKind::BitOrAssignment => op!(Value::bitor),
            AssignKind::BitXorAssignment => op!(Value::bitxor),
            AssignKind::PrefixIncrement => prefix!(Value::add),
            AssignKind::PostfixIncrement => postfix!(Value::add),
            AssignKind::PrefixDecrement => prefix!(Value::sub),
            AssignKind::PostfixDecrement => postfix!(Value::sub),
        };

        Ok(None)
    }

    pub fn dynamicpropertyaccess<'sc, 'vm>(
        mut cx: DispatchContext<'sc, 'vm>,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let key = cx.pop_stack_rooted();

        let preserve_this = cx.fetch_and_inc_ip() == 1;

        let target = if preserve_this {
            cx.peek_stack()
        } else {
            cx.pop_stack_rooted()
        };

        let key = PropertyKey::from_value(&mut cx, key)?;

        let value = target.get_property(&mut cx, key)?;
        cx.stack.push(value);
        Ok(None)
    }

    pub fn ldlocalext<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let value = Value::External(cx.get_external(id.into()).clone());

        // Unbox external values such that any use will create a copy
        let value = value.unbox_external();

        cx.stack.push(value);
        Ok(None)
    }

    fn assign_to_external(sc: &mut LocalScope, handle: &Handle<ExternalValue>, value: Value) {
        let value = value.into_gc(sc);
        unsafe { ExternalValue::replace(handle, value) };
    }

    pub fn storelocalext<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let value = Value::External(cx.get_external(id.into()).clone()).unbox_external();
                let right = cx.pop_stack_rooted();
                let res = $op(&value, &right, &mut cx)?;
                let external = cx.scope.get_external(id.into()).unwrap().clone();
                assign_to_external(&mut cx, &external, res.clone());
                cx.stack.push(res);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let value = Value::External(cx.get_external(id.into()).clone()).unbox_external();
                let right = Value::number(1.0);
                let res = $op(&value, &right, &mut cx)?;
                let external = cx.scope.get_external(id.into()).unwrap().clone();
                assign_to_external(&mut cx, &external, res.clone());
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let value = Value::External(cx.get_external(id.into()).clone()).unbox_external();
                let right = Value::number(1.0);
                let res = $op(&value, &right, &mut cx)?;
                let external = cx.scope.get_external(id.into()).unwrap().clone();
                assign_to_external(&mut cx, &external, res);
                cx.stack.push(value);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let value = cx.pop_stack_rooted();
                let external = cx.scope.get_external(id.into()).unwrap().clone();
                assign_to_external(&mut cx, &external, value.clone());
                cx.stack.push(value);
            }
            AssignKind::AddAssignment => op!(Value::add),
            AssignKind::SubAssignment => op!(Value::sub),
            AssignKind::MulAssignment => op!(Value::mul),
            AssignKind::DivAssignment => op!(Value::div),
            AssignKind::RemAssignment => op!(Value::rem),
            AssignKind::PowAssignment => op!(Value::pow),
            AssignKind::ShlAssignment => op!(Value::bitshl),
            AssignKind::ShrAssignment => op!(Value::bitshr),
            AssignKind::UshrAssignment => op!(Value::bitushr),
            AssignKind::BitAndAssignment => op!(Value::bitand),
            AssignKind::BitOrAssignment => op!(Value::bitor),
            AssignKind::BitXorAssignment => op!(Value::bitxor),
            AssignKind::PrefixIncrement => prefix!(Value::add),
            AssignKind::PostfixIncrement => postfix!(Value::add),
            AssignKind::PrefixDecrement => prefix!(Value::sub),
            AssignKind::PostfixDecrement => postfix!(Value::sub),
        }

        Ok(None)
    }

    pub fn try_block<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let ip = cx.active_frame().ip;
        let catch_offset = cx.fetchw_and_inc_ip() as usize;
        let catch_ip = ip + catch_offset + 2;
        let frame_ip = cx.frames.len();

        cx.try_blocks.push(TryBlock { catch_ip, frame_ip });

        Ok(None)
    }

    pub fn try_end<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.try_blocks.pop();
        Ok(None)
    }

    pub fn throw<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        Err(cx.pop_stack())
    }

    pub fn type_of<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        // NOTE: Does not need to be rooted. We don't call into JS.
        let value = cx.pop_stack_rooted();
        cx.stack.push(value.type_of().as_value());
        Ok(None)
    }

    pub fn yield_<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack();
        Ok(Some(HandleResult::Yield(value)))
    }

    pub fn await_<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack();
        Ok(Some(HandleResult::Await(value)))
    }

    pub fn import_dyn<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();

        let _ret = match cx.params.dynamic_import_callback() {
            Some(cb) => cb(&mut cx, value)?,
            None => throw!(cx, Error, "Dynamic imports are disabled for this context"),
        };

        // TODO: dynamic imports are currently statements, making them useless
        // TODO: make them an expression and push ret on stack

        Ok(None)
    }

    pub fn import_static<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let ty = StaticImportKind::from_repr(cx.fetch_and_inc_ip()).expect("Invalid import kind");
        let local_id = cx.fetchw_and_inc_ip();
        let path_id = cx.fetchw_and_inc_ip();

        let path = cx.string_constant(path_id.into());

        let value = match cx.params.static_import_callback() {
            Some(cb) => cb(&mut cx, ty, &path)?,
            None => throw!(cx, Error, "Static imports are disabled for this context."),
        };

        cx.set_local(local_id.into(), value.into());

        Ok(None)
    }

    pub fn export_default<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        // NOTE: Does not need to be rooted. Storing it in frame state counts as being rooted.
        let value = cx.pop_stack_rooted();
        let frame = cx.active_frame_mut();

        match &mut frame.state {
            FrameState::Module(module) => {
                module.default = Some(value.into());
            }
            _ => throw!(cx, Error, "Export is only available at the top level in modules"),
        }

        Ok(None)
    }

    pub fn export_named<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
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

                    let value = cx.global.clone().get_property(&mut cx, ident.as_ref().into())?;

                    (value, ident)
                }
                _ => unreachable!(),
            };

            let frame = cx.active_frame_mut();
            match &mut frame.state {
                FrameState::Module(exports) => exports.named.push((ident, value.into())),
                _ => throw!(cx, Error, "Export is only available at the top level in modules"),
            }
        }

        Ok(None)
    }

    pub fn debugger<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        if let Some(cb) = cx.params().debugger_callback() {
            cb(&mut cx)?;
        }

        Ok(None)
    }

    pub fn this<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let this = cx
            .frames
            .iter()
            .rev()
            .find_map(|f| f.this.as_ref())
            .cloned()
            .unwrap_or_else(|| Value::Object(cx.global.clone()));

        cx.stack.push(this);
        Ok(None)
    }

    pub fn global_this<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let global = cx.global.clone();
        cx.stack.push(Value::Object(global));
        Ok(None)
    }

    pub fn super_<'sc, 'vm>(cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        throw!(cx, SyntaxError, "`super` keyword unexpected in this context");
    }

    pub fn undef<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::undefined());
        Ok(None)
    }

    pub fn infinity<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::number(f64::INFINITY));
        Ok(None)
    }

    pub fn nan<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::number(f64::NAN));
        Ok(None)
    }

    pub fn call_symbol_iterator<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let symbol_iterator = cx.statics.symbol_iterator.clone();
        let iterable = value.get_property(&mut cx, PropertyKey::Symbol(symbol_iterator))?;
        let iterator = iterable.apply(&mut cx, value, Vec::new())?;
        cx.stack.push(iterator);
        Ok(None)
    }

    pub fn call_for_in_iterator<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();

        let keys = match value {
            Value::Object(obj) => obj.own_keys()?,
            Value::External(obj) => obj.own_keys()?,
            _ => Vec::new(),
        }
        .into_iter()
        .map(PropertyValue::static_default)
        .collect();

        let keys = Array::from_vec(&cx, keys);
        let keys = cx.register(keys);
        let iter = ArrayIterator::new(&mut cx, Value::Object(keys))?;
        let iter = cx.register(iter);
        cx.stack.push(Value::Object(iter));
        Ok(None)
    }

    pub fn delete_property_dynamic<'sc, 'vm>(
        mut cx: DispatchContext<'sc, 'vm>,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let (property, target) = cx.pop_stack2_rooted();
        let key = PropertyKey::from_value(&mut cx, property)?;
        let value = target.delete_property(&mut cx, key)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value.root(cx.scope), Value::Undefined(..));
        cx.stack.push(Value::Boolean(did_delete));
        Ok(None)
    }

    pub fn delete_property_static<'sc, 'vm>(
        mut cx: DispatchContext<'sc, 'vm>,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let target = cx.pop_stack_rooted();
        let cid = cx.fetchw_and_inc_ip();
        let con = cx.identifier_constant(cid.into());
        let key = PropertyKey::from(con.as_ref());
        let value = target.delete_property(&mut cx, key)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value.root(cx.scope), Value::Undefined(..));
        cx.stack.push(Value::Boolean(did_delete));
        Ok(None)
    }

    pub fn switch<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let case_count = cx.fetchw_and_inc_ip();
        let has_default = cx.fetch_and_inc_ip() == 1;

        let switch_expr = cx.pop_stack_rooted();

        let mut target_ip = None;

        for _ in 0..case_count {
            let case_value = cx.pop_stack_rooted();
            let case_offset = cx.fetchw_and_inc_ip() as usize;
            let ip = cx.active_frame().ip;

            let is_eq = switch_expr.strict_eq(&case_value, cx.scope)?.to_boolean()?;
            let has_matching_case = target_ip.is_some();

            if is_eq && !has_matching_case {
                target_ip = Some(ip + case_offset);
            }
        }

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

    pub fn objdestruct<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let count = cx.fetchw_and_inc_ip();
        let obj = cx.pop_stack_rooted();

        for _ in 0..count {
            let loc_id = cx.fetchw_and_inc_ip();
            let ident_id = cx.fetchw_and_inc_ip();

            let id = cx.number_constant(loc_id.into()) as usize;
            let ident = cx.identifier_constant(ident_id.into());

            let prop = obj.get_property(&mut cx, PropertyKey::from(ident.as_ref()))?;
            cx.set_local(id, prop.into());
        }

        Ok(None)
    }

    pub fn arraydestruct<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let count = cx.fetchw_and_inc_ip();
        let array = cx.pop_stack_rooted();

        for i in 0..count {
            let loc_id = cx.fetchw_and_inc_ip();

            let id = cx.number_constant(loc_id.into()) as usize;

            let prop = array.get_property(&mut cx, PropertyKey::from(i.to_string().as_ref()))?;
            cx.set_local(id, prop.into());
        }

        Ok(None)
    }

    pub fn intrinsic_op<'sc, 'vm>(mut cx: DispatchContext<'sc, 'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let op = IntrinsicOperation::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! lr_as_num_spec {
            () => {{
                // Unrooted is technically fine here, nothing can trigger a GC cycle
                // OK to remove if it turns out to be a useful opt
                let (left, right) = cx.pop_stack2_rooted();
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
                cx.stack.push(Value::number(((l as i64 as i32) $op (r as i64 as i32)) as f64));
            }};
        }
        macro_rules! bin_op_u64 {
            ($op:tt) => {{
                let (l, r) = lr_as_num_spec!();
                cx.stack.push(Value::number(((l as i64 as u32) $op (r as i64 as u32)) as f64));
            }};
        }

        macro_rules! bin_op_to_bool {
            ($op:tt) => {{
                let (l, r) = lr_as_num_spec!();
                cx.stack.push(Value::Boolean(l $op r));
            }};
        }

        macro_rules! postfix {
            ($op:tt) => {{
                let id = cx.fetch_and_inc_ip();
                let local = match cx.get_local(id.into()) {
                    Value::Number(n) => n,
                    _ => unreachable!(),
                };
                cx.set_local(id.into(), Value::number(local.0 $op 1.0).into());
                cx.stack.push(Value::Number(local));
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
                cx.set_local(id.into(), new.clone().into());
                cx.stack.push(new);
            }
            }};
        }

        macro_rules! bin_op_numl_constr {
            ($op:tt) => {{
                let left = match cx.pop_stack_rooted() {
                    Value::Number(n) => n.0,
                    _ => unreachable!(),
                };
                let right = cx.fetch_and_inc_ip() as f64;
                cx.stack.push(Value::Boolean(left $op right));
            }};
        }

        macro_rules! bin_op_numl_constr_n {
            ($op:tt, $ty:ty) => {{
                let left = match cx.pop_stack_rooted() {
                    Value::Number(n) => n.0,
                    _ => unreachable!(),
                };
                let mut right_bytes: [u8; <$ty>::BITS as usize / 8] = [0; <$ty>::BITS as usize / 8];
                for byte in right_bytes.iter_mut() {
                    *byte = cx.fetch_and_inc_ip();
                }
                let right = <$ty>::from_ne_bytes(right_bytes) as f64;
                cx.stack.push(Value::Boolean(left $op right));
            }};
        }

        macro_rules! fn_call {
            ($fun:ident, $k:ident, $v:ident) => {{
                let argc = cx.fetch_and_inc_ip();
                let args = cx.pop_stack_many(argc.into()).collect::<Vec<_>>();
                let fun = cx.statics.$fun.clone();

                if unlikely(!cx.builtins_purity()) {
                    for arg in &args {
                        cx.scope.add_value(arg.clone());
                    }

                    // TODO: don't warn here but when purity was violated
                    warn!("missed spec call due to impurity");
                    // Builtins impure, fallback to slow dynamic property lookup
                    let k = cx
                        .global
                        .clone()
                        .get_property(&mut cx, PropertyKey::from(stringify!($k)))?;
                    let fun = k.get_property(&mut cx, PropertyKey::from(stringify!($v)))?;
                    let result = fun.apply(&mut cx, Value::undefined(), args)?;
                    cx.stack.push(result);
                } else {
                    // Fastpath: call builtin directly
                    // TODO: should we add to externals?
                    let result = fun.apply(&mut cx, Value::undefined(), args)?;
                    cx.stack.push(result);
                }
            }};
        }

        match op {
            IntrinsicOperation::AddNumLR => bin_op!(Add::add),
            IntrinsicOperation::SubNumLR => bin_op!(Sub::sub),
            IntrinsicOperation::MulNumLR => bin_op!(Mul::mul),
            IntrinsicOperation::DivNumLR => bin_op!(Div::div),
            IntrinsicOperation::RemNumLR => bin_op!(Rem::rem),
            IntrinsicOperation::PowNumLR => bin_op!(f64::powf),
            IntrinsicOperation::GtNumLR => bin_op_to_bool!(>),
            IntrinsicOperation::GeNumLR => bin_op_to_bool!(>=),
            IntrinsicOperation::LtNumLR => bin_op_to_bool!(<),
            IntrinsicOperation::LeNumLR => bin_op_to_bool!(<=),
            IntrinsicOperation::EqNumLR => bin_op_to_bool!(==),
            IntrinsicOperation::NeNumLR => bin_op_to_bool!(!=),
            IntrinsicOperation::BitOrNumLR => bin_op_i64!(|),
            IntrinsicOperation::BitXorNumLR => bin_op_i64!(^),
            IntrinsicOperation::BitAndNumLR => bin_op_i64!(&),
            IntrinsicOperation::BitShlNumLR => bin_op_i64!(<<),
            IntrinsicOperation::BitShrNumLR => bin_op_i64!(>>),
            IntrinsicOperation::BitUshrNumLR => bin_op_u64!(>>),
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
            IntrinsicOperation::Exp => fn_call!(math_exp, Math, exp),
            IntrinsicOperation::Log2 => fn_call!(math_log2, Math, log2),
            IntrinsicOperation::Expm1 => fn_call!(math_expm1, Math, expm1),
            IntrinsicOperation::Cbrt => fn_call!(math_cbrt, Math, cbrt),
            IntrinsicOperation::Clz32 => fn_call!(math_clz32, Math, clz32),
            IntrinsicOperation::Atanh => fn_call!(math_atanh, Math, atanh),
            IntrinsicOperation::Atan2 => fn_call!(math_atan2, Math, atan2),
            IntrinsicOperation::Round => fn_call!(math_round, Math, round),
            IntrinsicOperation::Acosh => fn_call!(math_acosh, Math, acosh),
            IntrinsicOperation::Abs => fn_call!(math_abs, Math, abs),
            IntrinsicOperation::Sinh => fn_call!(math_sinh, Math, sinh),
            IntrinsicOperation::Sin => fn_call!(math_sin, Math, sin),
            IntrinsicOperation::Ceil => fn_call!(math_ceil, Math, ceil),
            IntrinsicOperation::Tan => fn_call!(math_tan, Math, tan),
            IntrinsicOperation::Trunc => fn_call!(math_trunc, Math, trunc),
            IntrinsicOperation::Asinh => fn_call!(math_asinh, Math, asinh),
            IntrinsicOperation::Log10 => fn_call!(math_log10, Math, log10),
            IntrinsicOperation::Asin => fn_call!(math_asin, Math, asin),
            IntrinsicOperation::Random => fn_call!(math_random, Math, random),
            IntrinsicOperation::Log1p => fn_call!(math_log1p, Math, log1p),
            IntrinsicOperation::Sqrt => fn_call!(math_sqrt, Math, sqrt),
            IntrinsicOperation::Atan => fn_call!(math_atan, Math, atan),
            IntrinsicOperation::Cos => fn_call!(math_cos, Math, cos),
            IntrinsicOperation::Tanh => fn_call!(math_tanh, Math, tanh),
            IntrinsicOperation::Log => fn_call!(math_log, Math, log),
            IntrinsicOperation::Floor => fn_call!(math_floor, Math, floor),
            IntrinsicOperation::Cosh => fn_call!(math_cosh, Math, cosh),
            IntrinsicOperation::Acos => fn_call!(math_acos, Math, acos),
        }

        Ok(None)
    }
}

pub fn handle(vm: &mut Vm, instruction: Instruction) -> Result<Option<HandleResult>, Unrooted> {
    // TODO: rework this
    // let scope_ref = cx.scope as *const LocalScope;
    // cx.externals.add(scope_ref, refs);
    let mut scope = vm.scope();
    let cx = DispatchContext::new(&mut scope);
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
        Instruction::StaticPropAssign => handlers::staticpropertyassign(cx),
        Instruction::DynamicPropAssign => handlers::dynamicpropertyassign(cx),
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
        Instruction::CallForInIterator => handlers::call_for_in_iterator(cx),
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
