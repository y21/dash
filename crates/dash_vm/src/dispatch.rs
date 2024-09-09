#![allow(clippy::needless_lifetimes)] // for now

use dash_log::warn;
use std::ops::{Deref, DerefMut};
use std::vec::Drain;

use crate::frame::Frame;
use crate::localscope::LocalScope;
use crate::value::string::JsString;
use crate::value::{ExternalValue, Root, Unrooted};

use super::value::Value;
use super::Vm;
use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::instruction::Instruction;

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

pub struct DispatchContext<'vm> {
    scope: LocalScope<'vm>,
}

impl<'vm> DispatchContext<'vm> {
    pub fn new(scope: LocalScope<'vm>) -> Self {
        Self { scope }
    }

    pub fn get_local(&mut self, index: usize) -> Value {
        self.scope
            .get_local(index)
            .expect("Bytecode attempted to reference invalid local")
    }

    pub fn get_external(&mut self, index: usize) -> &ExternalValue {
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
        self.scope.pop_stack_unwrap().root(&mut self.scope)
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

        let left = left.root(&mut self.scope);
        let right = right.root(&mut self.scope);

        let result = fun(&left, &right, self)?;
        self.stack.push(result);
        Ok(None)
    }

    pub fn active_frame(&self) -> &Frame {
        self.frames
            .last()
            .expect("Dispatch Context attempted to reference missing frame")
    }

    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    pub fn active_frame_mut(&mut self) -> &mut Frame {
        self.frames
            .last_mut()
            .expect("Dispatch Context attempted to reference missing frame")
    }

    pub fn constant(&self, index: usize) -> &Constant {
        &self.active_frame().function.constants[index]
    }

    pub fn identifier_constant(&self, index: usize) -> JsString {
        self.constant(index)
            .as_identifier()
            .expect("Bytecode attempted to reference invalid identifier constant")
            .into()
    }

    pub fn string_constant(&self, index: usize) -> JsString {
        self.constant(index)
            .as_string()
            .expect("Bytecode attempted to reference invalid string constant")
            .into()
    }

    pub fn number_constant(&self, index: usize) -> f64 {
        self.constant(index)
            .as_number()
            .expect("Bytecode attempted to reference invalid number constant")
    }
}

impl<'vm> Deref for DispatchContext<'vm> {
    type Target = LocalScope<'vm>;
    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl<'vm> DerefMut for DispatchContext<'vm> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}

mod extract {
    use std::convert::Infallible;
    use std::marker::PhantomData;

    use dash_middle::compiler::{ArrayMemberKind, ExportPropertyKind, ObjectMemberKind};
    use dash_middle::iterator_with::IteratorWith;

    use crate::gc::handle::Handle;
    use crate::value::object::{PropertyKey, PropertyValue};
    use crate::value::ops::conversions::ValueConversion;
    use crate::value::string::JsString;
    use crate::value::{Unrooted, Value};

    use super::DispatchContext;

    #[derive(Debug)]
    pub struct BackwardSequence<T> {
        index: usize,
        len: usize,
        _p: PhantomData<T>,
    }

    impl<T> BackwardSequence<T> {
        pub fn new_u16(cx: &mut DispatchContext<'_>) -> Self {
            let len = cx.fetchw_and_inc_ip();
            Self {
                index: 0,
                len: len as usize,
                _p: PhantomData,
            }
        }
        pub fn from_len(len: usize) -> Self {
            Self {
                index: 0,
                len,
                _p: PhantomData,
            }
        }
    }

    /// A sequence with extra capability to go forwards.
    #[derive(Debug)]
    pub struct ForwardSequence<T> {
        back: BackwardSequence<T>,
        stack_index: usize,
    }

    impl<T> ForwardSequence<T> {
        pub fn from_len(cx: &mut DispatchContext<'_>, iter_len: usize, stack_len: usize) -> Self {
            Self {
                back: BackwardSequence::from_len(iter_len),
                stack_index: cx.stack.len() - stack_len,
            }
        }
    }

    impl<'vm, T: ExtractBack> IteratorWith<&mut DispatchContext<'vm>> for BackwardSequence<T> {
        type Item = Result<T, T::Exception>;

        fn next(&mut self, cx: &mut DispatchContext<'vm>) -> Option<Self::Item> {
            if self.index == self.len {
                None
            } else {
                let item = T::extract(cx);
                self.index += 1;
                Some(item)
            }
        }
    }

    pub trait FrontIteratorWith<Args> {
        type Item;

        fn next_front(&mut self, args: Args) -> Option<Self::Item>;
    }
    impl<'vm, T: ExtractFront> FrontIteratorWith<&mut DispatchContext<'vm>> for ForwardSequence<T> {
        type Item = Result<T, T::Error>;
        fn next_front(&mut self, cx: &mut DispatchContext<'vm>) -> Option<Self::Item> {
            if self.back.index == self.back.len {
                None
            } else {
                let item = T::extract_front(self, cx);
                self.back.index += 1;
                Some(item)
            }
        }
    }

    pub trait ExtractBack: Sized {
        /// A note on errors: even though quite often errors are technically possible in implementations,
        /// we'll still use `Infallible`, because they're relying on bytecode invariants
        /// that, if they fail, indicate a bug elsewhere so there is no point in
        /// considering them errors that need to be handled.
        ///
        /// JS Exceptions on the other hand use `type Error = Value;` because they must be propagated
        type Exception;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception>;
    }

    pub trait ExtractFront: Sized {
        type Error;

        /// Extracts the value from the "front", as opposed to popping it off the back.
        /// The implementation is allowed to reorder the stack (e.g. via `swap_remove`)
        /// insofar everything behind the sequence is unaffected.
        fn extract_front<U>(seq: &mut ForwardSequence<U>, cx: &mut DispatchContext<'_>) -> Result<Self, Self::Error>;
    }

    #[derive(Debug)]
    pub enum ObjectProperty {
        Static { key: PropertyKey, value: PropertyValue },
        Getter { key: PropertyKey, value: Handle },
        Setter { key: PropertyKey, value: Handle },
        Spread(Value),
    }

    pub struct IdentW(pub JsString);

    impl ExtractBack for IdentW {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            let id = cx.fetchw_and_inc_ip();
            Ok(Self(cx.identifier_constant(id.into())))
        }
    }

    pub struct NumberWConstant(pub f64);

    impl ExtractBack for NumberWConstant {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            let id = cx.fetchw_and_inc_ip();
            Ok(Self(cx.number_constant(id.into())))
        }
    }

    pub struct Object(pub Handle);
    impl ExtractBack for Object {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            match cx.pop_stack_rooted() {
                Value::Object(o) => Ok(Self(o)),
                _ => panic!("stack top must contain an object"),
            }
        }
    }

    impl ExtractBack for ObjectMemberKind {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(ObjectMemberKind::from_repr(cx.fetch_and_inc_ip()).unwrap())
        }
    }

    impl ExtractBack for Value {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(cx.pop_stack_rooted())
        }
    }
    impl ExtractFront for Value {
        type Error = Infallible;

        fn extract_front<U>(seq: &mut ForwardSequence<U>, cx: &mut DispatchContext<'_>) -> Result<Self, Self::Error> {
            seq.stack_index += 1;
            let value = cx.stack[seq.stack_index - 1].clone();
            cx.scope.add_value(value.clone());
            Ok(value)
        }
    }

    /// Convenience function for infallibly extracting a `T`
    pub fn extract<T: ExtractBack<Exception = Infallible>>(cx: &mut DispatchContext<'_>) -> T {
        match T::extract(cx) {
            Ok(v) => v,
        }
    }

    /// Convenience function for infallibly extracting a `T`
    pub fn extract_front<T: ExtractFront<Error = Infallible>, U>(
        seq: &mut ForwardSequence<U>,
        cx: &mut DispatchContext<'_>,
    ) -> T {
        match T::extract_front(seq, cx) {
            Ok(v) => v,
        }
    }

    impl<E, A: ExtractBack<Exception = E>, B: ExtractBack<Exception = E>> ExtractBack for (A, B) {
        type Exception = E;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok((A::extract(cx)?, B::extract(cx)?))
        }
    }

    impl ExtractBack for ObjectProperty {
        type Exception = Value;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(match extract(cx) {
                ObjectMemberKind::Getter => {
                    let key = extract::<IdentW>(cx).0;
                    let value = extract::<Object>(cx).0;
                    Self::Getter { key: key.into(), value }
                }
                ObjectMemberKind::Setter => {
                    let key = extract::<IdentW>(cx).0;
                    let value = extract::<Object>(cx).0;
                    Self::Setter { key: key.into(), value }
                }
                ObjectMemberKind::Static => {
                    let key = extract::<IdentW>(cx).0;
                    let value = extract(cx);

                    Self::Static {
                        key: key.into(),
                        value: PropertyValue::static_default(value),
                    }
                }
                ObjectMemberKind::Dynamic => {
                    let key = extract(cx);
                    let value = extract(cx);

                    Self::Static {
                        key: PropertyKey::from_value(&mut cx.scope, key)?,
                        value: PropertyValue::static_default(value),
                    }
                }
                ObjectMemberKind::DynamicGetter => {
                    let key = extract(cx);
                    let value = extract::<Object>(cx).0;

                    Self::Getter {
                        key: PropertyKey::from_value(&mut cx.scope, key)?,
                        value,
                    }
                }
                ObjectMemberKind::DynamicSetter => {
                    let key = extract(cx);
                    let value = extract::<Object>(cx).0;

                    Self::Setter {
                        key: PropertyKey::from_value(&mut cx.scope, key)?,
                        value,
                    }
                }
                ObjectMemberKind::Spread => Self::Spread(extract(cx)),
            })
        }
    }

    #[derive(Debug)]
    pub enum ArrayElement {
        Single(Value),
        Spread(Value, usize),
        Hole(usize),
    }

    impl ExtractFront for ArrayElement {
        type Error = Value;

        fn extract_front<U>(seq: &mut ForwardSequence<U>, cx: &mut DispatchContext<'_>) -> Result<Self, Self::Error> {
            Ok(match extract::<ArrayMemberKind>(cx) {
                ArrayMemberKind::Item => ArrayElement::Single(extract_front(seq, cx)),
                ArrayMemberKind::Spread => {
                    let value: Value = extract_front(seq, cx);
                    // TODO: make this work for array-like values, not just arrays, by calling @@iterator on it
                    let len = value.length_of_array_like(&mut cx.scope)?;
                    ArrayElement::Spread(value, len)
                }
                ArrayMemberKind::Empty => {
                    let count = cx.fetch_and_inc_ip();
                    ArrayElement::Hole(count.into())
                }
            })
        }
    }

    impl ExtractBack for ArrayMemberKind {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(ArrayMemberKind::from_repr(cx.fetch_and_inc_ip()).unwrap())
        }
    }

    pub struct LocalW(pub Value);
    impl ExtractBack for LocalW {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            let local_id = cx.fetchw_and_inc_ip();
            Ok(Self(cx.get_local(local_id.into())))
        }
    }

    impl ExtractBack for ExportPropertyKind {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(Self::from_repr(cx.fetch_and_inc_ip()).unwrap())
        }
    }

    pub struct ExportProperty(pub Unrooted, pub JsString);
    impl ExtractBack for ExportProperty {
        type Exception = Unrooted;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(match extract(cx) {
                ExportPropertyKind::Local => {
                    let local = extract::<LocalW>(cx);
                    let ident = extract::<IdentW>(cx);
                    Self(local.0.into(), ident.0)
                }
                ExportPropertyKind::Global => {
                    let ident = extract::<IdentW>(cx).0;
                    let value = cx.global().get_property(&mut cx.scope, ident.into())?;
                    Self(value, ident)
                }
            })
        }
    }

    impl<E, T: ExtractBack<Exception = E>> ExtractBack for Option<T> {
        type Exception = E;
        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            match cx.fetch_and_inc_ip() {
                0 => Ok(None),
                1 => Ok(Some(T::extract(cx)?)),
                _ => unreachable!(),
            }
        }
    }

    impl ExtractBack for u16 {
        type Exception = Infallible;
        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(cx.fetchw_and_inc_ip())
        }
    }
}

mod handlers {
    use dash_middle::compiler::instruction::{AssignKind, IntrinsicOperation};
    use dash_middle::compiler::{FunctionCallMetadata, StaticImportKind};
    use dash_middle::interner::sym;
    use dash_middle::iterator_with::{InfallibleIteratorWith, IteratorWith};
    use handlers::extract::{extract, ForwardSequence, FrontIteratorWith};
    use hashbrown::hash_map::Entry;
    use if_chain::if_chain;
    use smallvec::SmallVec;
    use std::ops::{Add, ControlFlow, Div, Mul, Rem, Sub};

    use crate::frame::{FrameState, TryBlock};
    use crate::throw;
    use crate::util::unlikely;
    use crate::value::array::{Array, ArrayIterator, Element};
    use crate::value::function::user::UserFunction;
    use crate::value::function::{adjust_stack_from_flat_call, Function, FunctionKind};
    use crate::value::object::{NamedObject, Object, ObjectMap, PropertyKey, PropertyValue, PropertyValueKind};
    use crate::value::ops::conversions::ValueConversion;
    use crate::value::ops::equality;
    use crate::value::primitive::Number;

    use self::extract::{ArrayElement, BackwardSequence, ExportProperty, IdentW, NumberWConstant, ObjectProperty};

    use super::*;

    fn constant_instruction<'vm>(mut cx: DispatchContext<'vm>, idx: usize) -> Result<(), Value> {
        let constant = cx.constant(idx);

        let value = Value::from_constant(constant.clone(), &mut cx);
        cx.stack.push(value);
        Ok(())
    }

    pub fn constant<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        constant_instruction(cx, id as usize)?;
        Ok(None)
    }

    pub fn constantw<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        constant_instruction(cx, id as usize)?;
        Ok(None)
    }

    pub fn add<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::add)
    }

    pub fn sub<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::sub)
    }

    pub fn mul<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::mul)
    }

    pub fn div<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::div)
    }

    pub fn rem<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::rem)
    }

    pub fn pow<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::pow)
    }

    pub fn bitor<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitor)
    }

    pub fn bitxor<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitxor)
    }

    pub fn bitand<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitand)
    }

    pub fn bitshl<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitshl)
    }

    pub fn bitshr<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitshr)
    }

    pub fn bitushr<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitushr)
    }

    pub fn bitnot<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.bitnot(&mut cx)?;
        cx.stack.push(result);
        Ok(None)
    }

    pub fn objin<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|property, target, sc| {
            let property = property.to_js_string(sc)?;
            let found = target
                .for_each_prototype(sc, |sc, target| {
                    let contains = target
                        .own_keys(sc)?
                        .iter()
                        .any(|v| matches!(v, Value::String(s) if *s == property));

                    if contains {
                        Ok(ControlFlow::Break(()))
                    } else {
                        Ok(ControlFlow::Continue(()))
                    }
                })?
                .is_break();

            Ok(Value::Boolean(found))
        })
    }

    pub fn instanceof<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let (source, target) = cx.pop_stack2_rooted();

        let is_instanceof = source.instanceof(&target, &mut cx).map(Value::Boolean)?;
        cx.stack.push(is_instanceof);
        Ok(None)
    }

    pub fn lt<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::lt(l, r, sc).map(Value::Boolean))
    }

    pub fn le<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::le(l, r, sc).map(Value::Boolean))
    }

    pub fn gt<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::gt(l, r, sc).map(Value::Boolean))
    }

    pub fn ge<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::ge(l, r, sc).map(Value::Boolean))
    }

    pub fn eq<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::eq(l, r, sc).map(Value::Boolean))
    }

    pub fn ne<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::ne(l, r, sc).map(Value::Boolean))
    }

    pub fn strict_eq<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, _| Ok(Value::Boolean(equality::strict_eq(l, r))))
    }

    pub fn strict_ne<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, _| Ok(Value::Boolean(equality::strict_ne(l, r))))
    }

    pub fn neg<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.to_number(&mut cx)?;
        cx.stack.push(Value::number(-result));
        Ok(None)
    }

    pub fn pos<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.to_number(&mut cx)?;
        cx.stack.push(Value::number(result));
        Ok(None)
    }

    pub fn not<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.not(&mut cx.scope);
        cx.stack.push(result);
        Ok(None)
    }

    pub fn pop<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.pop_stack();
        Ok(None)
    }

    pub fn delayed_ret(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack();
        cx.active_frame_mut().delayed_ret = Some(Ok(value));
        Ok(None)
    }

    pub fn finally_end<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let tc_depth = cx.fetchw_and_inc_ip();

        if let Some(ret) = cx.active_frame_mut().delayed_ret.take() {
            let ret = ret?.root(&mut cx.scope);
            let frame_ip = cx.frames.len();
            let enclosing_finally = cx
                .try_blocks
                .iter()
                .find_map(|tc| if tc.frame_ip == frame_ip { tc.finally_ip } else { None });

            if let Some(finally) = enclosing_finally {
                let lower_tcp = cx.try_blocks.len() - usize::from(tc_depth);
                drop(cx.try_blocks.drain(lower_tcp..));
                cx.active_frame_mut().ip = finally;
            } else {
                let this = cx.pop_frame();
                return Ok(ret_inner(cx, tc_depth, ret, this));
            }
        }
        Ok(None)
    }

    fn ret_inner(mut cx: DispatchContext<'_>, tc_depth: u16, value: Value, this: Frame) -> Option<HandleResult> {
        // Drain all try catch blocks that are in this frame.
        let lower_tcp = cx.try_blocks.len() - usize::from(tc_depth);
        drop(cx.try_blocks.drain(lower_tcp..));

        // Drain all the stack space from this frame
        drop(cx.stack.drain(this.sp..));

        match this.state {
            FrameState::Module(_) => {
                // Put it back on the frame stack, because we'll need it in Vm::execute_module
                cx.frames.push(this);
                Some(HandleResult::Return(Unrooted::new(value)))
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
                            None
                        } else {
                            Some(HandleResult::Return(Unrooted::new(this)))
                        }
                    }
                    else {
                        if is_flat_call {
                            cx.stack.push(value);
                            None
                        } else {
                            Some(HandleResult::Return(Unrooted::new(value)))
                        }
                    }
                }
            }
        }
    }

    pub fn ret<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let tc_depth = cx.fetchw_and_inc_ip();
        let value = cx.pop_stack_rooted();
        let this = cx.pop_frame();
        Ok(ret_inner(cx, tc_depth, value, this))
    }

    pub fn ldglobal<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let name = cx.identifier_constant(id.into());

        let value = match cx.global.as_any().downcast_ref::<NamedObject>() {
            Some(value) => match value.get_raw_property(name.into()) {
                Some(value) => value.kind().get_or_apply(&mut cx, Value::undefined())?,
                None => {
                    let name = name.res(&cx.scope).to_owned();
                    throw!(&mut cx, ReferenceError, "{} is not defined", name)
                }
            },
            None => cx.global.clone().get_property(&mut cx, name.into())?,
        };

        cx.push_stack(value);
        Ok(None)
    }

    pub fn storeglobal<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let name = cx.identifier_constant(id.into());
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let right = cx.pop_stack_rooted();
                let value = cx
                    .global
                    .clone()
                    .get_property(&mut cx, PropertyKey::String(name))
                    .root(&mut cx.scope)?;

                let res = $op(&value, &right, &mut cx)?;
                cx.global
                    .clone()
                    .set_property(&mut cx, name.into(), PropertyValue::static_default(res.clone()))?;
                cx.stack.push(res);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let value = cx
                    .global
                    .clone()
                    .get_property(&mut cx, PropertyKey::String(name))
                    .root(&mut cx.scope)?;
                let value = Value::number(value.to_number(&mut cx)?);

                let right = Value::number(1.0);
                let res = $op(&value, &right, &mut cx)?;
                cx.global
                    .clone()
                    .set_property(&mut cx, name.into(), PropertyValue::static_default(res.clone()))?;
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let value = cx
                    .global
                    .clone()
                    .get_property(&mut cx, PropertyKey::String(name))
                    .root(&mut cx.scope)?;
                let value = Value::number(value.to_number(&mut cx)?);

                let right = Value::number(1.0);
                let res = $op(&value, &right, &mut cx)?;
                cx.global
                    .clone()
                    .set_property(&mut cx, name.into(), PropertyValue::static_default(res))?;
                cx.stack.push(value);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let value = cx.pop_stack_rooted();

                cx.global
                    .clone()
                    .set_property(&mut cx, name.into(), PropertyValue::static_default(value.clone()))?;
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
    fn call_flat<'vm>(
        mut cx: DispatchContext<'vm>,
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
                let length = iterable.length_of_array_like(&mut cx.scope)?;

                let mut splice_args = SmallVec::<[_; 2]>::new();

                for i in 0..length {
                    let i = cx.scope.intern_usize(i);
                    let value = iterable.get_property(&mut cx, i.into())?.root(&mut cx.scope);
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

        let arguments = adjust_stack_from_flat_call(&mut cx, user_function, sp, argc);

        let mut frame = Frame::from_function(Some(this), user_function, is_constructor, true, arguments);
        frame.set_sp(sp);

        cx.pad_stack_for_frame(&frame);
        cx.try_push_frame(frame)?;

        Ok(None)
    }

    /// Fallback for callable values that are not "function objects"
    fn call_generic<'vm>(
        mut cx: DispatchContext<'vm>,
        callee: &Value,
        this: Value,
        argc: usize,
        is_constructor: bool,
        call_ip: u16,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let args = {
            let mut args = Vec::with_capacity(argc);

            let len = cx.fetch_and_inc_ip();
            let spread_indices: SmallVec<[_; 4]> = (0..len).map(|_| cx.fetch_and_inc_ip()).collect();

            let iter = cx.pop_stack_many(argc);

            if len == 0 {
                // Fast path for no spread arguments
                for value in iter {
                    args.push(value);
                }
            } else {
                let raw_args: SmallVec<[_; 4]> = iter.collect();
                let mut indices_iter = spread_indices.into_iter().peekable();

                for (index, value) in raw_args.into_iter().enumerate() {
                    if indices_iter.peek().is_some_and(|&v| usize::from(v) == index) {
                        let len = value.length_of_array_like(&mut cx.scope)?;
                        for i in 0..len {
                            let i = cx.scope.intern_usize(i);
                            let value = value.get_property(&mut cx, i.into())?.root(&mut cx.scope);
                            // NB: no need to push into `refs` since we already rooted it
                            args.push(value);
                        }
                        indices_iter.next();
                    } else {
                        args.push(value);
                    }
                }
            }

            args
        };

        cx.scope.add_many(&args);

        let ret = if is_constructor {
            callee.construct(&mut cx, this, args)?
        } else {
            callee.apply_with_debug(&mut cx, this, args, call_ip)?
        };

        // SAFETY: no need to root, we're directly pushing into the value stack which itself is a root
        cx.push_stack(ret);
        Ok(None)
    }

    pub fn call<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let call_ip = cx.active_frame().ip as u16 - 1;

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

        if let Some(function) = callee.downcast_ref::<Function>() {
            match function.kind() {
                FunctionKind::User(user) => call_flat(cx, &callee, this, function, user, argc, is_constructor),
                FunctionKind::Closure(closure) => {
                    let bound_this = closure.this.clone();
                    call_flat(cx, &callee, bound_this, function, &closure.fun, argc, is_constructor)
                }
                _ => call_generic(cx, &callee, this, argc, is_constructor, call_ip),
            }
        } else {
            call_generic(cx, &callee, this, argc, is_constructor, call_ip)
        }
    }

    pub fn jmpfalsep<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;

        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack_rooted();

        let jump = !value.is_truthy(&mut cx.scope);

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

    pub fn jmpfalsenp<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = !value.is_truthy(&mut cx.scope);

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

    pub fn jmptruep<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;

        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack_rooted();

        let jump = value.is_truthy(&mut cx.scope);

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

    pub fn jmptruenp<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = value.is_truthy(&mut cx.scope);

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

    pub fn jmpnullishp<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
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

    pub fn jmpnullishnp<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
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

    pub fn jmpundefinedp<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack_rooted();

        let jump = matches!(value, Value::Undefined(_));

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

    pub fn jmpundefinednp<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        #[cfg(feature = "jit")]
        let ip = cx.active_frame().ip;
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = matches!(value, Value::Null(_));

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

    pub fn jmp<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
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

    pub fn storelocal<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
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

    pub fn ldlocal<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let value = cx.get_local(id.into());

        cx.stack.push(value);
        Ok(None)
    }

    fn with_arraylit_elements(
        cx: &mut DispatchContext<'_>,
        len: usize,
        stack_values: usize,
        mut fun: impl FnMut(Element<PropertyValue>),
    ) -> Result<(), Unrooted> {
        let mut iter = ForwardSequence::<ArrayElement>::from_len(cx, len, stack_values);
        while let Some(element) = iter.next_front(cx) {
            match element? {
                ArrayElement::Single(value) => fun(Element::Value(PropertyValue::static_default(value))),
                ArrayElement::Spread(source, len) => {
                    for i in 0..len {
                        let i = cx.scope.intern_usize(i);

                        let value = source.get_property(&mut cx.scope, i.into())?.root(&mut cx.scope);
                        fun(Element::Value(PropertyValue::static_default(value)));
                    }
                }
                ArrayElement::Hole(count) => fun(Element::Hole { count }),
            }
        }
        let truncate_to = cx.stack.len() - stack_values;
        cx.stack.truncate(truncate_to);

        debug_assert!(iter.next_front(cx).is_none());
        Ok(())
    }

    fn arraylit_holey(cx: &mut DispatchContext<'_>, len: usize, stack_values: usize) -> Result<Array, Unrooted> {
        let mut new_elements = Vec::with_capacity(stack_values);
        with_arraylit_elements(cx, len, stack_values, |element| new_elements.push(element))?;
        Ok(Array::from_possibly_holey(&cx.scope, new_elements))
    }

    fn arraylit_dense(cx: &mut DispatchContext<'_>, len: usize) -> Result<Array, Unrooted> {
        // Dense implies len == stack_values
        let mut new_elements = Vec::with_capacity(len);
        with_arraylit_elements(cx, len, len, |element| match element {
            Element::Hole { .. } => unreachable!(),
            Element::Value(v) => new_elements.push(v),
        })?;
        Ok(Array::from_vec(&cx.scope, new_elements))
    }

    pub fn arraylit(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let len = cx.fetch_and_inc_ip() as usize;
        let stack_values = cx.fetch_and_inc_ip() as usize;
        // Split up into two functions as a non-holey array literal can be evaluated more efficiently
        let array = if len == stack_values {
            arraylit_dense(&mut cx, len)?
        } else {
            arraylit_holey(&mut cx, len, stack_values)?
        };

        let handle = cx.gc.register(array);
        cx.stack.push(Value::Object(handle));
        Ok(None)
    }

    pub fn objlit<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let mut iter = BackwardSequence::<ObjectProperty>::new_u16(&mut cx);

        let mut obj = ObjectMap::default();
        while let Some(property) = iter.next(&mut cx) {
            match property? {
                ObjectProperty::Static { key, value } => drop(obj.insert(key, value)),
                ObjectProperty::Getter { key, value } => match obj.entry(key) {
                    Entry::Occupied(mut entry) => match &mut entry.get_mut().kind {
                        PropertyValueKind::Static(_) => drop(entry.insert(PropertyValue::getter_default(value))),
                        PropertyValueKind::Trap { get, .. } => *get = Some(value),
                    },
                    Entry::Vacant(entry) => drop(entry.insert(PropertyValue::getter_default(value))),
                },
                ObjectProperty::Setter { key, value } => match obj.entry(key) {
                    Entry::Occupied(mut entry) => match &mut entry.get_mut().kind {
                        PropertyValueKind::Static(_) => drop(entry.insert(PropertyValue::setter_default(value))),
                        PropertyValueKind::Trap { set, .. } => *set = Some(value),
                    },
                    Entry::Vacant(entry) => drop(entry.insert(PropertyValue::setter_default(value))),
                },
                ObjectProperty::Spread(value) => {
                    if let Value::Object(object) = value {
                        for key in object.own_keys(&mut cx.scope)? {
                            let key = PropertyKey::from_value(&mut cx.scope, key)?;
                            let value = object.get_property(&mut cx, key.clone())?.root(&mut cx.scope);
                            obj.insert(key, PropertyValue::static_default(value));
                        }
                    }
                }
            }
        }

        let obj = NamedObject::with_values(&cx, obj);

        let handle = cx.gc.register(obj);
        cx.stack.push(handle.into());

        Ok(None)
    }

    pub fn assign_properties(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let mut iter = BackwardSequence::<ObjectProperty>::new_u16(&mut cx);
        let target = cx.pop_stack_rooted();

        while let Some(property) = iter.next(&mut cx) {
            let property = property?;
            let is_getter = matches!(property, ObjectProperty::Getter { .. });

            match property {
                ObjectProperty::Static { key, value } => target.set_property(&mut cx.scope, key, value)?,
                ObjectProperty::Getter { key, value } | ObjectProperty::Setter { key, value } => {
                    let prop = target.get_property_descriptor(&mut cx.scope, key.clone())?;
                    let prop = match prop {
                        Some(mut prop) => {
                            if let PropertyValueKind::Trap { get, set } = &mut prop.kind {
                                if is_getter {
                                    *get = Some(value);
                                } else {
                                    *set = Some(value);
                                }
                            }
                            prop
                        }
                        None => {
                            if is_getter {
                                PropertyValue::getter_default(value)
                            } else {
                                PropertyValue::setter_default(value)
                            }
                        }
                    };

                    target.set_property(&mut cx.scope, key, prop)?;
                }
                ObjectProperty::Spread(_) => unimplemented!("spread operator in AssignProperties"),
            }
        }

        Ok(None)
    }

    pub fn staticpropertyaccess<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let ident = cx.identifier_constant(id.into());

        let preserve_this = cx.fetch_and_inc_ip() == 1;

        let target = if preserve_this {
            cx.peek_stack()
        } else {
            cx.pop_stack_rooted()
        };

        let value = target.get_property(&mut cx, ident.into())?;
        cx.push_stack(value);
        Ok(None)
    }

    pub fn staticpropertyassign<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();
        let id = cx.fetchw_and_inc_ip();
        let key = cx.identifier_constant(id.into());

        macro_rules! op {
            ($op:expr) => {{
                let (target, value) = cx.pop_stack2_new();

                let target = target.root(&mut cx.scope);
                let value = value.root(&mut cx.scope);

                let p = target.get_property(&mut cx, key.into())?.root(&mut cx.scope);
                let res = $op(&p, &value, &mut cx)?;

                target.set_property(&mut cx, key.into(), PropertyValue::static_default(res.clone()))?;
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let target = cx.pop_stack_rooted();
                let prop = target.get_property(&mut cx, key.into())?.root(&mut cx.scope);
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(&prop, &one, &mut cx)?;
                target.set_property(&mut cx, key.into(), PropertyValue::static_default(res))?;
                cx.stack.push(prop);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let target = cx.pop_stack_rooted();
                let prop = target.get_property(&mut cx, key.into())?.root(&mut cx.scope);
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(&prop, &one, &mut cx)?;
                target.set_property(&mut cx, key.into(), PropertyValue::static_default(res.clone()))?;
                cx.stack.push(res);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let (target, value) = cx.pop_stack2_new();
                let target = target.root(&mut cx.scope);
                let value = value.root(&mut cx.scope);
                target.set_property(&mut cx, key.into(), PropertyValue::static_default(value.clone()))?;
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

    pub fn dynamicpropertyassign<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let (target, value, key) = cx.pop_stack3_rooted();

                let key = PropertyKey::from_value(&mut cx, key)?;
                let prop = target.get_property(&mut cx, key.clone())?.root(&mut cx.scope);

                let result = $op(&prop, &value, &mut cx)?;

                target.set_property(&mut cx, key, PropertyValue::static_default(result.clone()))?;
                cx.stack.push(result);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let (target, key) = cx.pop_stack2_rooted();
                let key = PropertyKey::from_value(&mut cx, key)?;
                let prop = target.get_property(&mut cx, key.clone())?.root(&mut cx.scope);
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
                let prop = target.get_property(&mut cx, key.clone())?.root(&mut cx.scope);
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

    pub fn dynamicpropertyaccess<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let key = cx.pop_stack_rooted();

        let preserve_this = cx.fetch_and_inc_ip() == 1;

        let target = if preserve_this {
            cx.peek_stack()
        } else {
            cx.pop_stack_rooted()
        };

        let key = PropertyKey::from_value(&mut cx, key)?;

        let value = target.get_property(&mut cx, key)?;
        cx.push_stack(value);
        Ok(None)
    }

    pub fn ldlocalext<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let value = Value::External(cx.get_external(id.into()).clone());

        // Unbox external values such that any use will create a copy
        let value = value.unbox_external();

        cx.stack.push(value);
        Ok(None)
    }

    fn assign_to_external(handle: &ExternalValue, value: Value) {
        unsafe { ExternalValue::replace(handle, value) };
    }

    pub fn storelocalext<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetch_and_inc_ip();
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let value = Value::External(cx.get_external(id.into()).clone()).unbox_external();
                let right = cx.pop_stack_rooted();
                let res = $op(&value, &right, &mut cx)?;
                let external = cx.scope.get_external(id.into()).unwrap().clone();
                assign_to_external(&external, res.clone());
                cx.stack.push(res);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let value = Value::External(cx.get_external(id.into()).clone()).unbox_external();
                let right = Value::number(1.0);
                let res = $op(&value, &right, &mut cx)?;
                let external = cx.scope.get_external(id.into()).unwrap().clone();
                assign_to_external(&external, res.clone());
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let value = Value::External(cx.get_external(id.into()).clone()).unbox_external();
                let right = Value::number(1.0);
                let res = $op(&value, &right, &mut cx)?;
                let external = cx.scope.get_external(id.into()).unwrap().clone();
                assign_to_external(&external, res);
                cx.stack.push(value);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let value = cx.pop_stack_rooted();
                let external = cx.scope.get_external(id.into()).unwrap().clone();
                assign_to_external(&external, value.clone());
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

    pub fn try_block<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let mut compute_dist_ip = || {
            let distance = extract::<Option<u16>>(&mut cx)?;
            let ip = cx.active_frame().ip;
            Some(ip + distance as usize)
        };

        let catch_ip = compute_dist_ip();
        let finally_ip = compute_dist_ip();
        let frame_ip = cx.frames.len();

        cx.try_blocks.push(TryBlock {
            catch_ip,
            finally_ip,
            frame_ip,
        });

        Ok(None)
    }

    pub fn try_end<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.try_blocks.pop();
        Ok(None)
    }

    pub fn throw<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        Err(cx.pop_stack())
    }

    pub fn type_of<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        cx.stack.push(value.type_of().as_value());
        Ok(None)
    }

    pub fn type_of_ident<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let constant = cx.identifier_constant(id.into());
        let prop = cx
            .global
            .clone()
            .get_property(&mut cx.scope, constant.into())?
            .root(&mut cx.scope);

        cx.stack.push(prop.type_of().as_value());
        Ok(None)
    }

    pub fn yield_<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack();
        Ok(Some(HandleResult::Yield(value)))
    }

    pub fn await_<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack();
        Ok(Some(HandleResult::Await(value)))
    }

    pub fn import_dyn<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();

        let _ret = match cx.params.dynamic_import_callback() {
            Some(cb) => cb(&mut cx, value)?,
            None => throw!(cx, Error, "Dynamic imports are disabled for this context"),
        };

        // TODO: dynamic imports are currently statements, making them useless
        // TODO: make them an expression and push ret on stack

        Ok(None)
    }

    pub fn import_static<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let ty = StaticImportKind::from_repr(cx.fetch_and_inc_ip()).expect("Invalid import kind");
        let local_id = cx.fetchw_and_inc_ip();
        let path_id = cx.fetchw_and_inc_ip();

        let path = cx.string_constant(path_id.into());

        let value = match cx.params.static_import_callback() {
            Some(cb) => cb(&mut cx, ty, path)?,
            None => throw!(cx, Error, "Static imports are disabled for this context."),
        };

        cx.set_local(local_id.into(), value);

        Ok(None)
    }

    pub fn export_default<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        // NOTE: Does not need to be rooted. Storing it in frame state counts as being rooted.
        let value = cx.pop_stack();
        let frame = cx.active_frame_mut();

        match &mut frame.state {
            FrameState::Module(module) => {
                module.default = Some(value);
            }
            _ => throw!(cx, Error, "Export is only available at the top level in modules"),
        }

        Ok(None)
    }

    pub fn export_named<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let mut iter = BackwardSequence::<ExportProperty>::new_u16(&mut cx);
        while let Some(prop) = iter.next(&mut cx) {
            let ExportProperty(value, ident) = prop?;

            match &mut cx.active_frame_mut().state {
                FrameState::Module(exports) => exports.named.push((ident, value)),
                _ => throw!(cx, Error, "Export is only available at the top level in modules"),
            }
        }
        Ok(None)
    }

    pub fn debugger<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        if let Some(cb) = cx.params().debugger_callback() {
            cb(&mut cx)?;
        }

        Ok(None)
    }

    pub fn this<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
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

    pub fn global_this<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let global = cx.global.clone();
        cx.stack.push(Value::Object(global));
        Ok(None)
    }

    pub fn super_<'vm>(cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        throw!(cx, SyntaxError, "`super` keyword unexpected in this context");
    }

    pub fn undef<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::undefined());
        Ok(None)
    }

    pub fn infinity<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::number(f64::INFINITY));
        Ok(None)
    }

    pub fn nan<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::number(f64::NAN));
        Ok(None)
    }

    pub fn call_symbol_iterator<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let symbol_iterator = cx.statics.symbol_iterator.clone();
        let iterable = value
            .get_property(&mut cx, PropertyKey::Symbol(symbol_iterator))?
            .root(&mut cx.scope);
        let iterator = iterable.apply(&mut cx, value, Vec::new())?;
        cx.push_stack(iterator);
        Ok(None)
    }

    pub fn call_for_in_iterator<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();

        let keys = match value {
            Value::Object(obj) => obj.own_keys(&mut cx.scope)?,
            Value::External(obj) => obj.own_keys(&mut cx.scope)?,
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

    pub fn delete_property_dynamic<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let (property, target) = cx.pop_stack2_rooted();
        let key = PropertyKey::from_value(&mut cx, property)?;
        let value = target.delete_property(&mut cx, key)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value.root(&mut cx.scope), Value::Undefined(..));
        cx.stack.push(Value::Boolean(did_delete));
        Ok(None)
    }

    pub fn delete_property_static<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let target = cx.pop_stack_rooted();
        let cid = cx.fetchw_and_inc_ip();
        let con = cx.identifier_constant(cid.into());
        let value = target.delete_property(&mut cx, con.into())?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value.root(&mut cx.scope), Value::Undefined(..));
        cx.stack.push(Value::Boolean(did_delete));
        Ok(None)
    }

    pub fn objdestruct<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let rest_id = match cx.fetchw_and_inc_ip() {
            u16::MAX => None,
            n => Some(n),
        };
        let obj = cx.pop_stack_rooted();

        let mut idents = Vec::new();

        let mut iter = BackwardSequence::<(NumberWConstant, IdentW)>::new_u16(&mut cx);
        while let Some((NumberWConstant(id), IdentW(ident))) = iter.next_infallible(&mut cx) {
            if rest_id.is_some() {
                idents.push(ident);
            }

            let prop = obj.get_property(&mut cx, ident.into())?;
            cx.set_local(id as usize, prop);
        }

        if let Some(rest_id) = rest_id {
            let keys = obj
                .own_keys(&mut cx.scope)?
                .into_iter()
                .filter_map(|s| match s {
                    Value::String(s) => (!idents.contains(&s)).then_some(s),
                    _ => unreachable!("own_keys returned non-string"),
                })
                .collect::<Vec<_>>();

            let rest = NamedObject::new(&cx.scope);
            let rest = cx.scope.register(rest);
            for key in keys {
                let value = obj.get_property(&mut cx.scope, key.into())?.root(&mut cx.scope);
                rest.set_property(&mut cx.scope, key.into(), PropertyValue::static_default(value))?;
            }

            cx.set_local(rest_id.into(), Value::Object(rest).into());
        }

        Ok(None)
    }

    pub fn arraydestruct<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let array = cx.pop_stack_rooted();

        let mut iter = BackwardSequence::<Option<NumberWConstant>>::new_u16(&mut cx).enumerate();

        while let Some((i, id)) = iter.next_infallible(&mut cx) {
            if let Some(NumberWConstant(id)) = id {
                let id = id as usize;
                let key = cx.scope.intern_usize(i);
                let prop = array.get_property(&mut cx.scope, key.into())?;
                cx.set_local(id, prop);
            }
        }

        Ok(None)
    }

    pub fn intrinsic_op<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
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

        fn logical_op_numl_u32r_n<F: FnOnce(f64, f64) -> bool>(mut cx: DispatchContext<'_>, f: F) {
            let vm: &mut Vm = &mut cx;

            let Some(value @ &mut Value::Number(Number(left))) = vm.stack.last_mut() else {
                unreachable!()
            };
            let frame = vm.frames.last_mut().unwrap();
            let ip = frame.ip;
            let right = frame
                .function
                .buffer
                .with(|buf| u32::from_ne_bytes(buf[ip..ip + 4].try_into().unwrap()) as f64);
            frame.ip += 4;

            *value = Value::Boolean(f(left, right));
        }

        macro_rules! fn_call {
            ($fun:ident, $k:expr, $v:expr) => {{
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
                        .get_property(&mut cx, $k.into())?
                        .root(&mut cx.scope);
                    let fun = k.get_property(&mut cx, $v.into())?.root(&mut cx.scope);
                    let result = fun.apply(&mut cx, Value::undefined(), args)?;
                    cx.push_stack(result);
                } else {
                    // Fastpath: call builtin directly
                    // TODO: should we add to externals?
                    let result = fun.apply(&mut cx, Value::undefined(), args)?;
                    cx.push_stack(result);
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
            IntrinsicOperation::GtNumLConstR32 => logical_op_numl_u32r_n(cx, |l, r| l > r),
            IntrinsicOperation::GeNumLConstR32 => logical_op_numl_u32r_n(cx, |l, r| l >= r),
            IntrinsicOperation::LtNumLConstR32 => logical_op_numl_u32r_n(cx, |l, r| l < r),
            IntrinsicOperation::LeNumLConstR32 => logical_op_numl_u32r_n(cx, |l, r| l <= r),
            IntrinsicOperation::Exp => fn_call!(math_exp, sym::Math, sym::exp),
            IntrinsicOperation::Log2 => fn_call!(math_log2, sym::Math, sym::log2),
            IntrinsicOperation::Expm1 => fn_call!(math_expm1, sym::Math, sym::expm1),
            IntrinsicOperation::Cbrt => fn_call!(math_cbrt, sym::Math, sym::cbrt),
            IntrinsicOperation::Clz32 => fn_call!(math_clz32, sym::Math, sym::clz32),
            IntrinsicOperation::Atanh => fn_call!(math_atanh, sym::Math, sym::atanh),
            IntrinsicOperation::Atan2 => fn_call!(math_atan2, sym::Math, sym::atan2),
            IntrinsicOperation::Round => fn_call!(math_round, sym::Math, sym::round),
            IntrinsicOperation::Acosh => fn_call!(math_acosh, sym::Math, sym::acosh),
            IntrinsicOperation::Abs => fn_call!(math_abs, sym::Math, sym::abs),
            IntrinsicOperation::Sinh => fn_call!(math_sinh, sym::Math, sym::sinh),
            IntrinsicOperation::Sin => fn_call!(math_sin, sym::Math, sym::sin),
            IntrinsicOperation::Ceil => fn_call!(math_ceil, sym::Math, sym::ceil),
            IntrinsicOperation::Tan => fn_call!(math_tan, sym::Math, sym::tan),
            IntrinsicOperation::Trunc => fn_call!(math_trunc, sym::Math, sym::trunc),
            IntrinsicOperation::Asinh => fn_call!(math_asinh, sym::Math, sym::asinh),
            IntrinsicOperation::Log10 => fn_call!(math_log10, sym::Math, sym::log10),
            IntrinsicOperation::Asin => fn_call!(math_asin, sym::Math, sym::asin),
            IntrinsicOperation::Random => fn_call!(math_random, sym::Math, sym::random),
            IntrinsicOperation::Log1p => fn_call!(math_log1p, sym::Math, sym::log1p),
            IntrinsicOperation::Sqrt => fn_call!(math_sqrt, sym::Math, sym::sqrt),
            IntrinsicOperation::Atan => fn_call!(math_atan, sym::Math, sym::atan),
            IntrinsicOperation::Cos => fn_call!(math_cos, sym::Math, sym::cos),
            IntrinsicOperation::Tanh => fn_call!(math_tanh, sym::Math, sym::tanh),
            IntrinsicOperation::Log => fn_call!(math_log, sym::Math, sym::log),
            IntrinsicOperation::Floor => fn_call!(math_floor, sym::Math, sym::floor),
            IntrinsicOperation::Cosh => fn_call!(math_cosh, sym::Math, sym::cosh),
            IntrinsicOperation::Acos => fn_call!(math_acos, sym::Math, sym::acos),
        }

        Ok(None)
    }

    pub fn arguments<'vm>(mut cx: DispatchContext<'vm>) -> Result<Option<HandleResult>, Unrooted> {
        let arguments = cx
            .active_frame()
            .arguments
            .clone()
            .expect("`arguments` was never set despite being referenced in bytecode");
        cx.stack.push(Value::Object(arguments));
        Ok(None)
    }
}

pub fn handle(vm: &mut Vm, instruction: Instruction) -> Result<Option<HandleResult>, Unrooted> {
    let cx = DispatchContext::new(vm.scope());
    match instruction {
        Instruction::Add => handlers::add(cx),
        Instruction::Sub => handlers::sub(cx),
        Instruction::Mul => handlers::mul(cx),
        Instruction::Div => handlers::div(cx),
        Instruction::Rem => handlers::rem(cx),
        Instruction::Pow => handlers::pow(cx),
        Instruction::Gt => handlers::gt(cx),
        Instruction::Ge => handlers::ge(cx),
        Instruction::Lt => handlers::lt(cx),
        Instruction::Le => handlers::le(cx),
        Instruction::Eq => handlers::eq(cx),
        Instruction::Ne => handlers::ne(cx),
        Instruction::Pop => handlers::pop(cx),
        Instruction::LdLocal => handlers::ldlocal(cx),
        Instruction::LdGlobal => handlers::ldglobal(cx),
        Instruction::Constant => handlers::constant(cx),
        Instruction::ConstantW => handlers::constantw(cx),
        Instruction::Pos => handlers::pos(cx),
        Instruction::Neg => handlers::neg(cx),
        Instruction::TypeOf => handlers::type_of(cx),
        Instruction::TypeOfGlobalIdent => handlers::type_of_ident(cx),
        Instruction::BitNot => handlers::bitnot(cx),
        Instruction::Not => handlers::not(cx),
        Instruction::StoreLocal => handlers::storelocal(cx),
        Instruction::StoreGlobal => handlers::storeglobal(cx),
        Instruction::Ret => handlers::ret(cx),
        Instruction::Call => handlers::call(cx),
        Instruction::JmpFalseP => handlers::jmpfalsep(cx),
        Instruction::Jmp => handlers::jmp(cx),
        Instruction::StaticPropAccess => handlers::staticpropertyaccess(cx),
        Instruction::DynamicPropAccess => handlers::dynamicpropertyaccess(cx),
        Instruction::ArrayLit => handlers::arraylit(cx),
        Instruction::ObjLit => handlers::objlit(cx),
        Instruction::This => handlers::this(cx),
        Instruction::StaticPropAssign => handlers::staticpropertyassign(cx),
        Instruction::DynamicPropAssign => handlers::dynamicpropertyassign(cx),
        Instruction::LdLocalExt => handlers::ldlocalext(cx),
        Instruction::StoreLocalExt => handlers::storelocalext(cx),
        Instruction::StrictEq => handlers::strict_eq(cx),
        Instruction::StrictNe => handlers::strict_ne(cx),
        Instruction::Try => handlers::try_block(cx),
        Instruction::TryEnd => handlers::try_end(cx),
        Instruction::FinallyEnd => handlers::finally_end(cx),
        Instruction::Throw => handlers::throw(cx),
        Instruction::Yield => handlers::yield_(cx),
        Instruction::JmpFalseNP => handlers::jmpfalsenp(cx),
        Instruction::JmpTrueP => handlers::jmptruep(cx),
        Instruction::JmpTrueNP => handlers::jmptruenp(cx),
        Instruction::JmpNullishP => handlers::jmpnullishp(cx),
        Instruction::JmpNullishNP => handlers::jmpnullishnp(cx),
        Instruction::JmpUndefinedNP => handlers::jmpundefinednp(cx),
        Instruction::JmpUndefinedP => handlers::jmpundefinedp(cx),
        Instruction::BitOr => handlers::bitor(cx),
        Instruction::BitXor => handlers::bitxor(cx),
        Instruction::BitAnd => handlers::bitand(cx),
        Instruction::BitShl => handlers::bitshl(cx),
        Instruction::BitShr => handlers::bitshr(cx),
        Instruction::BitUshr => handlers::bitushr(cx),
        Instruction::ObjIn => handlers::objin(cx),
        Instruction::InstanceOf => handlers::instanceof(cx),
        Instruction::ImportDyn => handlers::import_dyn(cx),
        Instruction::ImportStatic => handlers::import_static(cx),
        Instruction::ExportDefault => handlers::export_default(cx),
        Instruction::ExportNamed => handlers::export_named(cx),
        Instruction::Debugger => handlers::debugger(cx),
        Instruction::Global => handlers::global_this(cx),
        Instruction::Super => handlers::super_(cx),
        Instruction::Arguments => handlers::arguments(cx),
        Instruction::Undef => handlers::undef(cx),
        Instruction::Await => handlers::await_(cx),
        Instruction::Nan => handlers::nan(cx),
        Instruction::Infinity => handlers::infinity(cx),
        Instruction::IntrinsicOp => handlers::intrinsic_op(cx),
        Instruction::CallSymbolIterator => handlers::call_symbol_iterator(cx),
        Instruction::CallForInIterator => handlers::call_for_in_iterator(cx),
        Instruction::DeletePropertyStatic => handlers::delete_property_static(cx),
        Instruction::DeletePropertyDynamic => handlers::delete_property_dynamic(cx),
        Instruction::ObjDestruct => handlers::objdestruct(cx),
        Instruction::ArrayDestruct => handlers::arraydestruct(cx),
        Instruction::AssignProperties => handlers::assign_properties(cx),
        Instruction::DelayedReturn => handlers::delayed_ret(cx),
        Instruction::Nop => Ok(None),
        _ => unimplemented!("{:?}", instruction),
    }
}
