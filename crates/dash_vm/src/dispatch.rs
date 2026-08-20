use dash_middle::compiler::constant::{ConstantPool, NumberConstant, SymbolConstant};
use dash_middle::compiler::external::ExternalId;
use dash_middle::compiler::scope::BackLocalId;
use std::ops::{Deref, DerefMut};

use crate::frame::Frame;
use crate::localscope::LocalScope;
use crate::value::string::JsString;
use crate::value::{ExternalValue, Root, Unrooted};

use super::Vm;
use super::value::Value;
use dash_middle::compiler::instruction::Instruction;

#[derive(Debug)]
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

    pub fn get_local(&mut self, index: BackLocalId) -> Value {
        self.scope
            .get_local(index)
            .expect("Bytecode attempted to reference invalid local")
    }

    pub fn get_external(&mut self, index: ExternalId) -> ExternalValue {
        self.scope.get_external(index)
    }

    pub fn pop_frame(&mut self) -> Frame {
        self.frames.pop()
    }

    pub fn pop_stack(&mut self) -> Unrooted {
        self.scope.pop_stack_unwrap()
    }

    pub fn pop_stack_rooted(&mut self) -> Value {
        self.scope.pop_stack_unwrap().root(&mut self.scope)
    }

    pub fn peek_stack(&mut self) -> Unrooted {
        Unrooted::new(
            *self
                .stack
                .last()
                .expect("Bytecode attempted to peek stack value, but nothing was on the stack"),
        )
    }

    fn pop_stack_const<const N: usize>(&mut self) -> [Unrooted; N] {
        assert!(self.stack.len() >= N);
        let mut arr: [Unrooted; N] = std::array::from_fn(|_| Unrooted::new(self.stack.pop().unwrap()));
        arr.reverse();
        arr
    }

    pub fn pop_stack2_rooted(&mut self) -> (Value, Value) {
        let [a, b] = self.pop_stack_const();
        (a.root(&mut self.scope), b.root(&mut self.scope))
    }

    pub fn pop_stack3_rooted(&mut self) -> (Value, Value, Value) {
        let [a, b, c] = self.pop_stack_const();
        (
            a.root(&mut self.scope),
            b.root(&mut self.scope),
            c.root(&mut self.scope),
        )
    }

    pub fn evaluate_binary_with_scope<F>(&mut self, fun: F) -> Result<Option<HandleResult>, Unrooted>
    where
        F: Fn(Value, Value, &mut LocalScope) -> Result<Value, Value>,
    {
        let (left, right) = self.pop_stack2_rooted();

        let result = fun(left, right, self)?;
        self.stack.push(result);
        Ok(None)
    }

    pub fn constants(&self) -> &ConstantPool {
        self.frames.current_constants()
    }
}

impl<'vm> Deref for DispatchContext<'vm> {
    type Target = LocalScope<'vm>;
    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl DerefMut for DispatchContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}

mod extract {
    use std::convert::Infallible;
    use std::marker::PhantomData;

    use dash_middle::compiler::constant::{NumberConstant, SymbolConstant};
    use dash_middle::compiler::scope::BackLocalId;
    use dash_middle::compiler::{ArrayMemberKind, ExportPropertyKind, FunctionCallKind, ObjectMemberKind};
    use dash_middle::iterator_with::IteratorWith;

    use crate::gc::ObjectId;
    use crate::value::object::PropertyValue;
    use crate::value::ops::conversions::ValueConversion;
    use crate::value::propertykey::{PropertyKey, ToPropertyKey};
    use crate::value::string::JsString;
    use crate::value::{Unpack, Unrooted, Value, ValueKind};

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
        Getter { key: PropertyKey, value: ObjectId },
        Setter { key: PropertyKey, value: ObjectId },
        Spread(Value),
    }

    pub struct IdentW(pub JsString);

    impl ExtractBack for IdentW {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            let id = cx.fetchw_and_inc_ip();
            Ok(Self(cx.constants().symbols[SymbolConstant(id)].into()))
        }
    }

    pub struct NumberWConstant(pub f64);

    impl ExtractBack for NumberWConstant {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            let id = cx.fetchw_and_inc_ip();
            Ok(Self(cx.constants().numbers[NumberConstant(id)]))
        }
    }

    pub struct Object(pub ObjectId);
    impl ExtractBack for Object {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            match cx.pop_stack_rooted().unpack() {
                ValueKind::Object(o) => Ok(Self(o)),
                _ => panic!("stack top must contain an object"),
            }
        }
    }

    impl ExtractFront for Object {
        type Error = Infallible;

        fn extract_front<U>(seq: &mut ForwardSequence<U>, cx: &mut DispatchContext<'_>) -> Result<Self, Self::Error> {
            let value: Value = extract_front(seq, cx);
            match value.unpack() {
                ValueKind::Object(o) => Ok(Self(o)),
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
            let value = cx.stack[seq.stack_index - 1];
            cx.scope.add(value);
            Ok(value)
        }
    }

    impl ExtractBack for bool {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(cx.fetch_and_inc_ip() == 1)
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

    macro_rules! tupl_impl {
        ($($($param:ident)*),*) => {
            $(
                impl<E $(, $param : ExtractBack<Exception = E>)*> ExtractBack for ($($param),*) {
                    type Exception = E;

                    fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
                        Ok((
                            $(
                                <$param>::extract(cx)?
                            ),*
                        ))
                    }
                }
            )*
        };
    }
    tupl_impl! {
        A B,
        A B C
    }

    impl ExtractFront for ObjectProperty {
        type Error = Value;

        fn extract_front<U>(seq: &mut ForwardSequence<U>, cx: &mut DispatchContext<'_>) -> Result<Self, Self::Error> {
            Ok(match extract(cx) {
                ObjectMemberKind::Getter => {
                    let key = extract::<IdentW>(cx).0;
                    let Object(value) = extract_front(seq, cx);
                    Self::Getter {
                        key: key.to_key(&mut cx.scope),
                        value,
                    }
                }
                ObjectMemberKind::Setter => {
                    let key = extract::<IdentW>(cx).0;
                    let Object(value) = extract_front(seq, cx);
                    Self::Setter {
                        key: key.to_key(&mut cx.scope),
                        value,
                    }
                }
                ObjectMemberKind::Static => {
                    let key = extract::<IdentW>(cx).0;
                    let value = extract_front(seq, cx);

                    Self::Static {
                        key: key.to_key(&mut cx.scope),
                        value: PropertyValue::static_default(value),
                    }
                }
                ObjectMemberKind::Dynamic => {
                    let key = extract_front(seq, cx);
                    let value = extract_front(seq, cx);

                    Self::Static {
                        key: PropertyKey::from_value(&mut cx.scope, key)?,
                        value: PropertyValue::static_default(value),
                    }
                }
                ObjectMemberKind::DynamicGetter => {
                    let key = extract_front(seq, cx);
                    let Object(value) = extract_front(seq, cx);

                    Self::Getter {
                        key: PropertyKey::from_value(&mut cx.scope, key)?,
                        value,
                    }
                }
                ObjectMemberKind::DynamicSetter => {
                    let key = extract_front(seq, cx);
                    let Object(value) = extract_front(seq, cx);

                    Self::Setter {
                        key: PropertyKey::from_value(&mut cx.scope, key)?,
                        value,
                    }
                }
                ObjectMemberKind::Spread => Self::Spread(extract_front(seq, cx)),
            })
        }
    }

    #[derive(Debug)]
    pub enum ArrayElement {
        Single(Value),
        Spread(Value, usize),
        Hole(u32),
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
            Ok(Self(cx.get_local(BackLocalId(local_id))))
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
                    let value = cx.global().get_property(ident.to_key(&mut cx.scope), &mut cx.scope)?;
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

    impl ExtractBack for FunctionCallKind {
        type Exception = Infallible;
        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(Self::from_repr(cx.fetch_and_inc_ip()).unwrap())
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct LoopHotnessByte(u8);

    impl LoopHotnessByte {
        const COUNTER_MASK: u8 = 0b01111111;
        const DISABLED_MASK: u8 = !Self::COUNTER_MASK;

        pub fn try_increment(self) -> Option<Self> {
            let counter = self.0 & Self::COUNTER_MASK;
            if counter == Self::COUNTER_MASK {
                None
            } else {
                Some(Self(counter + 1))
            }
        }

        pub fn is_disabled(self) -> bool {
            self.0 & Self::DISABLED_MASK != 0
        }

        pub fn disable(self) -> Self {
            Self(self.0 | Self::DISABLED_MASK)
        }

        pub fn raw(self) -> u8 {
            self.0
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct LoopBackjumpData {
        pub offset: i16,
        pub hotness: LoopHotnessByte,
    }

    impl ExtractBack for LoopBackjumpData {
        type Exception = Infallible;

        fn extract(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            let [hotness, off1, off2] = cx.frames.fetch_n_and_inc_ip::<3>();
            let offset = i16::from_ne_bytes([off1, off2]);
            Ok(Self {
                offset,
                hotness: LoopHotnessByte(hotness),
            })
        }
    }
}

mod handlers {
    use dash_middle::compiler::constant::{BooleanConstant, FunctionConstant, RegexConstant};
    use dash_middle::compiler::external::{External, PossiblyExternalId};
    use dash_middle::compiler::instruction::{AssignKind, IntrinsicOperation};
    use dash_middle::compiler::{FunctionCallKind, StaticImportKind};
    use dash_middle::interner::sym;
    use dash_middle::iterator_with::{InfallibleIteratorWith, IteratorWith};
    use dash_middle::parser::statement::{Asyncness, FunctionKind as ParserFunctionKind};
    use handlers::extract::{ForwardSequence, FrontIteratorWith, extract};
    use if_chain::if_chain;
    use smallvec::SmallVec;
    use std::assert_matches;
    use std::ops::{Add, ControlFlow, Div, Mul, Rem, Sub};
    use std::rc::Rc;

    use crate::dispatch::extract::LoopBackjumpData;
    use crate::frame::{FrameState, Ip, Sp, TryBlock};
    use crate::jit::JitReturn;
    use crate::util::unlikely;
    use crate::value::array::table::ArrayTable;
    use crate::value::array::{Array, ArrayIterator};
    use crate::value::function::args::CallArgs;
    use crate::value::function::r#async::AsyncFunction;
    use crate::value::function::closure::Closure;
    use crate::value::function::generator::GeneratorFunction;
    use crate::value::function::user::UserFunction;
    use crate::value::function::{Function, FunctionKind, adjust_stack_from_flat_call, this_for_new_target};
    use crate::value::object::{Object, OrdObject, OwnKeysMode, PropertyValue, PropertyValueKind, This, ThisKind};
    use crate::value::ops::conversions::ValueConversion;
    use crate::value::ops::equality;
    use crate::value::primitive::Number;
    use crate::value::propertykey::{PropertyKey, ToPropertyKey};
    use crate::value::regex::RegExp;
    use crate::value::{Unpack, ValueKind};
    use crate::{jit, throw};

    use self::extract::{ArrayElement, BackwardSequence, ExportProperty, IdentW, NumberWConstant, ObjectProperty};

    use super::*;

    pub fn string_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let sym = JsString::from(cx.constants().symbols[SymbolConstant(id)]);
        cx.push_stack(Value::string(sym).into());
        Ok(None)
    }

    pub fn boolean_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let b = cx.constants().booleans[BooleanConstant(id)];
        cx.push_stack(Value::boolean(b).into());
        Ok(None)
    }

    pub fn number_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let n = cx.constants().numbers[NumberConstant(id)];
        cx.push_stack(Value::number(n).into());
        Ok(None)
    }

    pub fn regex_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let (regex, source) = &cx.constants().regexes[RegexConstant(id)];

        let regex = RegExp::new(regex.clone(), JsString::from(*source), &cx.scope);
        let regex = cx.scope.register(regex);
        cx.push_stack(Value::object(regex).into());
        Ok(None)
    }

    pub fn null_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.push_stack(Value::null().into());
        Ok(None)
    }

    pub fn undefined_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.push_stack(Value::undefined().into());
        Ok(None)
    }

    pub fn function_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        fn register_function_externals(
            function: &dash_middle::compiler::constant::Function,
            sc: &mut LocalScope<'_>,
        ) -> Vec<ExternalValue> {
            let mut externals = Vec::new();

            for External { id } in function.externals.iter().copied() {
                let value = match id {
                    PossiblyExternalId::Local(id) => {
                        let value = sc.get_local_raw(id).expect("Referenced local not found");

                        // "Box up" the value at the local slot by wrapping it in a `Value::External`,
                        // if it isn't already.
                        match value.unpack() {
                            ValueKind::External(value) => value,
                            _ => {
                                let ext_id = sc.register(value);
                                sc.set_local(id, Value::external(ext_id).into());
                                ExternalValue::new(sc, ext_id)
                            }
                        }
                    }
                    PossiblyExternalId::External(id) => sc.get_external(id),
                };

                externals.push(value);
            }

            externals
        }

        let id = cx.fetchw_and_inc_ip();
        let fun = Rc::clone(&cx.constants().functions[FunctionConstant(id)]);

        let externals = register_function_externals(&fun, &mut cx.scope);

        let name = fun.name.map(Into::into);
        let ty = fun.ty;

        let fun = UserFunction::new(fun, externals.into());

        let kind = match ty {
            ParserFunctionKind::Function(Asyncness::Yes) => FunctionKind::Async(AsyncFunction::new(fun)),
            ParserFunctionKind::Function(Asyncness::No) => FunctionKind::User(fun),
            ParserFunctionKind::Arrow => FunctionKind::Closure(Closure {
                fun,
                this: cx.scope.frames.current_this(),
            }),
            ParserFunctionKind::Generator => FunctionKind::Generator(GeneratorFunction::new(fun)),
        };

        let function = Function::builder(kind).maybe_name(name).alloc_in_scope(&mut cx.scope);
        cx.push_stack(Value::object(function).into());

        Ok(None)
    }

    pub fn add(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::add)
    }

    pub fn sub(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::sub)
    }

    pub fn mul(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::mul)
    }

    pub fn div(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::div)
    }

    pub fn rem(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::rem)
    }

    pub fn pow(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::pow)
    }

    pub fn bitor(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitor)
    }

    pub fn bitxor(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitxor)
    }

    pub fn bitand(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitand)
    }

    pub fn bitshl(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitshl)
    }

    pub fn bitshr(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitshr)
    }

    pub fn bitushr(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(Value::bitushr)
    }

    pub fn bitnot(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.bitnot(&mut cx)?;
        cx.stack.push(result);
        Ok(None)
    }

    pub fn objin(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|property, target, sc| {
            let property = property.to_js_string(sc)?;
            let found = target
                .for_each_prototype(sc, |sc, target| {
                    let contains = target
                        .own_keys(sc, OwnKeysMode::All)?
                        .iter()
                        .any(|v| matches!(v.unpack(), ValueKind::String(s) if s == property));

                    if contains {
                        Ok(ControlFlow::Break(()))
                    } else {
                        Ok(ControlFlow::Continue(()))
                    }
                })?
                .is_break();

            Ok(Value::boolean(found))
        })
    }

    pub fn instanceof(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let (source, target) = cx.pop_stack2_rooted();

        let is_instanceof = source.instanceof(&target, &mut cx).map(Value::boolean)?;
        cx.stack.push(is_instanceof);
        Ok(None)
    }

    pub fn lt(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::lt(l, r, sc).map(Value::boolean))
    }

    pub fn le(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::le(l, r, sc).map(Value::boolean))
    }

    pub fn gt(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::gt(l, r, sc).map(Value::boolean))
    }

    pub fn ge(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::ge(l, r, sc).map(Value::boolean))
    }

    pub fn eq(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::eq(l, r, sc).map(Value::boolean))
    }

    pub fn ne(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, sc| equality::ne(l, r, sc).map(Value::boolean))
    }

    pub fn strict_eq(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, _| Ok(Value::boolean(equality::strict_eq(l, r))))
    }

    pub fn strict_ne(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.evaluate_binary_with_scope(|l, r, _| Ok(Value::boolean(equality::strict_ne(l, r))))
    }

    pub fn neg(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.to_number(&mut cx)?;
        cx.stack.push(Value::number(-result));
        Ok(None)
    }

    pub fn pos(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.to_number(&mut cx)?;
        cx.stack.push(Value::number(result));
        Ok(None)
    }

    pub fn not(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let result = value.not(&mut cx.scope);
        cx.stack.push(result);
        Ok(None)
    }

    pub fn pop(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.pop_stack();
        Ok(None)
    }

    pub fn delayed_ret(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack();
        cx.frames.set_delayed_ret(Some(Ok(value)));
        Ok(None)
    }

    pub fn finally_end(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let tc_depth = cx.fetchw_and_inc_ip();

        if let Some(ret) = cx.frames.take_delayed_ret() {
            let ret = ret?.root(&mut cx.scope);
            let frame_idx = cx.frames.current_id();
            // NOTE: the try block was re-pushed in handle_rt_error
            let enclosing_finally = cx
                .try_blocks
                .iter()
                .find_map(|tc| if tc.frame_idx == frame_idx { tc.finally_ip } else { None });

            if let Some(finally) = enclosing_finally {
                let lower_tcp = cx.try_blocks.len() - usize::from(tc_depth);
                drop(cx.try_blocks.drain(lower_tcp..));
                cx.frames.set_ip(finally);
            } else {
                let this = cx.pop_frame();
                return ret_inner(cx, tc_depth, ret, this);
            }
        }
        Ok(None)
    }

    fn ret_inner(
        mut cx: DispatchContext<'_>,
        tc_depth: u16,
        value: Value,
        this: Frame,
    ) -> Result<Option<HandleResult>, Unrooted> {
        // Drain all try catch blocks that are in this frame.
        let lower_tcp = cx.try_blocks.len() - usize::from(tc_depth);
        drop(cx.try_blocks.drain(lower_tcp..));

        // Drain all the stack space from this frame
        drop(cx.stack.drain(this.sp.0 as usize..));

        match this.state {
            FrameState::Module(_) => {
                // Put it back on the frame stack, because we'll need it in Vm::execute_module
                cx.frames.push(this).expect("frame was just popped");
                Ok(Some(HandleResult::Return(Unrooted::new(value))))
            }
            FrameState::Function {
                new_target,
                is_flat_call,
            } => {
                if_chain! {
                    if new_target.is_some() && !matches!(value.unpack(), ValueKind::Object(_) | ValueKind::External(_));
                    then {
                        let this = this.this.to_value(&mut cx.scope)?;
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

    pub fn ret(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let tc_depth = cx.fetchw_and_inc_ip();
        let value = cx.pop_stack_rooted();
        let this = cx.pop_frame();
        ret_inner(cx, tc_depth, value, this)
    }

    pub fn ldglobal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let name = JsString::from(cx.constants().symbols[SymbolConstant(id)]);

        let value = match cx.global.clone().extract::<OrdObject>(&cx.scope) {
            Some(value) => match value.get_own_property_descriptor(name.to_key(&mut cx.scope), &mut cx.scope)? {
                Some(value) => value.kind().get_or_apply(&mut cx, This::default())?,
                None => {
                    let name = name.res(&cx.scope).to_owned();
                    throw!(&mut cx, ReferenceError, "{} is not defined", name)
                }
            },
            None => cx.global.get_property(name.to_key(&mut cx.scope), &mut cx.scope)?,
        };

        cx.push_stack(value);
        Ok(None)
    }

    pub fn storeglobal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let name = JsString::from(cx.constants().symbols[SymbolConstant(id)]);
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let right = cx.pop_stack_rooted();
                let value = cx
                    .global
                    .clone()
                    .get_property(name.to_key(&mut cx.scope), &mut cx.scope)
                    .root(&mut cx.scope)?;

                let res = $op(value, right, &mut cx)?;
                cx.global.clone().set_property(
                    name.to_key(&mut cx.scope),
                    PropertyValue::static_default(res.clone()),
                    &mut cx.scope,
                )?;
                cx.stack.push(res);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let value = cx
                    .global
                    .clone()
                    .get_property(name.to_key(&mut cx.scope), &mut cx.scope)
                    .root(&mut cx.scope)?;
                let value = Value::number(value.to_number(&mut cx)?);

                let right = Value::number(1.0);
                let res = $op(value, right, &mut cx)?;
                cx.global.clone().set_property(
                    name.to_key(&mut cx.scope),
                    PropertyValue::static_default(res.clone()),
                    &mut cx.scope,
                )?;
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let value = cx
                    .global
                    .clone()
                    .get_property(name.to_key(&mut cx.scope), &mut cx.scope)
                    .root(&mut cx.scope)?;
                let value = Value::number(value.to_number(&mut cx)?);

                let right = Value::number(1.0);
                let res = $op(value, right, &mut cx)?;
                cx.global.clone().set_property(
                    name.to_key(&mut cx.scope),
                    PropertyValue::static_default(res),
                    &mut cx.scope,
                )?;
                cx.stack.push(value);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let value = cx.pop_stack_rooted();

                cx.global.clone().set_property(
                    name.to_key(&mut cx.scope),
                    PropertyValue::static_default(value),
                    &mut cx.scope,
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
    fn call_flat(
        mut cx: DispatchContext<'_>,
        callee: Value,
        this: This,
        _function: &Function,
        user_function: &UserFunction,
        mut argc: usize,
        kind: FunctionCallKind,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let sp_before_call = cx.stack.len() - argc;
        let ValueKind::Object(callee) = callee.unpack() else {
            unreachable!("guaranteed by caller")
        };

        let (this, new_target) = match kind {
            // new.target is always the callee in this codepath.
            FunctionCallKind::Constructor => {
                let this = if user_function.inner().has_extends_clause {
                    let ValueKind::Object(super_constructor) = callee.get_prototype(&mut cx.scope)?.unpack() else {
                        throw!(cx.scope, TypeError, "supertype constructor must be an object")
                    };

                    This::before_super(super_constructor)
                } else {
                    this_for_new_target(&mut cx.scope, callee)?
                };

                (this, Some(callee))
            }
            FunctionCallKind::Function => (this, None),
            FunctionCallKind::Super => {
                let this = if user_function.inner().has_extends_clause {
                    let ValueKind::Object(super_constructor) = callee.get_prototype(&mut cx.scope)?.unpack() else {
                        throw!(cx.scope, TypeError, "supertype constructor must be an object")
                    };

                    This::before_super(super_constructor)
                } else {
                    let new_target = cx.frames.current_state().new_target().unwrap();
                    this_for_new_target(&mut cx.scope, new_target)?
                };

                (this, cx.frames.current_state().new_target())
            }
        };

        let spread_arguments = cx.fetch_and_inc_ip();

        // If we have spread args, we need to "splice" values from iterables in.
        // This is hopefully rather "rare" (compared to regular call arguments),
        // so we can afford to do more work here in order to keep the common path fast.
        if spread_arguments > 0 {
            let spread_indices: SmallVec<[_; 4]> = (0..spread_arguments).map(|_| cx.fetch_and_inc_ip()).collect();
            let mut spread_count = 0;

            let mut splice_args = Vec::new();
            for spread_index in spread_indices {
                splice_args.clear();
                let adjusted_spread_index = (sp_before_call as isize + spread_index as isize + spread_count) as usize;

                let iterable = cx.stack[adjusted_spread_index];
                let length = iterable.length_of_array_like(&mut cx.scope)?;

                for i in 0..length {
                    let value = iterable
                        .get_property(i.to_key(&mut cx.scope), &mut cx.scope)?
                        .root(&mut cx.scope);
                    splice_args.push(value);
                }
                cx.stack.splice(
                    adjusted_spread_index..=adjusted_spread_index,
                    splice_args.iter().copied(),
                );

                spread_count += (length as isize) - 1;
            }

            argc = (argc as isize + spread_count) as usize;
        }

        // NOTE: since we are in a "flat" call,
        // we don't need to add objects to the external
        // reference list since they stay on the VM stack
        // and are reachable from there

        let arguments = adjust_stack_from_flat_call(&mut cx, user_function, sp_before_call, argc);

        let mut frame = Frame::from_function(this, user_function, new_target, true, arguments);
        frame.sp = Sp(sp_before_call as u32);

        cx.init_stack_for_frame(&frame);
        cx.try_push_frame(frame)?;

        Ok(None)
    }

    /// Fallback for callable values that are not "function objects"
    fn call_generic(
        mut cx: DispatchContext<'_>,
        callee: Value,
        this: This,
        argc: usize,
        function_call_kind: FunctionCallKind,
        call_ip: Ip,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let args = {
            let mut args = SmallVec::with_capacity(argc);

            let len = cx.fetch_and_inc_ip();
            let spread_indices: SmallVec<[_; 4]> = (0..len).map(|_| cx.fetch_and_inc_ip()).collect();

            let raw_args = cx.drain_stack_rooted(argc);

            if len == 0 {
                // Fast path for no spread arguments
                args.extend(raw_args);
            } else {
                let mut indices_iter = spread_indices.into_iter().peekable();
                let raw_args = raw_args.collect::<SmallVec<[Value; 3]>>();

                for (index, value) in raw_args.into_iter().enumerate() {
                    if indices_iter.peek().is_some_and(|&v| usize::from(v) == index) {
                        let len = value.length_of_array_like(&mut cx.scope)?;
                        for i in 0..len {
                            let value = value
                                .get_property(i.to_key(&mut cx.scope), &mut cx.scope)?
                                .root(&mut cx.scope);
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

        let ret = match function_call_kind {
            FunctionCallKind::Constructor => callee.construct(this, args.into(), &mut cx.scope)?,
            FunctionCallKind::Super => {
                let new_target = cx.frames.current_state().new_target().unwrap();
                callee.construct_with_target(this, args.into(), new_target, &mut cx.scope)?
            }
            FunctionCallKind::Function => callee.apply_with_debug(this, args.into(), call_ip, &mut cx.scope)?,
        };

        // SAFETY: no need to root, we're directly pushing into the value stack which itself is a root
        cx.push_stack(ret);
        Ok(None)
    }

    pub fn call(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        // FIXME: sketchy assumption
        let call_ip = cx.frames.current_ip() - 1;

        let argc = usize::from(cx.fetch_and_inc_ip());
        let has_this = extract::<bool>(&mut cx);
        let function_call_kind = extract::<FunctionCallKind>(&mut cx);

        let stack_len = cx.stack.len();
        let (callee, this) = if function_call_kind == FunctionCallKind::Super {
            let callee = match cx.frames.current_this().kind() {
                ThisKind::BeforeSuper { super_constructor } => Value::object(super_constructor),
                _ => throw!(
                    cx.scope,
                    TypeError,
                    "super() must be called exactly once in a subclass constructor"
                ),
            };
            (callee, This::default())
        } else if has_this {
            cx.stack[stack_len - argc - 2..].rotate_left(2);
            let (this, callee) = cx.pop_stack2_rooted();
            (callee, This::bound(this))
        } else {
            cx.stack[stack_len - argc - 1..].rotate_left(1);
            let callee = cx.pop_stack_rooted();
            (callee, This::default())
        };

        if let Some(function) = callee.unpack().downcast_ref::<Function>(&cx.scope) {
            match function.kind() {
                FunctionKind::User(user) => call_flat(cx, callee, this, function, user, argc, function_call_kind),
                FunctionKind::Closure(closure) => {
                    if function_call_kind == FunctionCallKind::Constructor {
                        throw!(cx.scope, TypeError, "closure cannot be called as a constructor")
                    }

                    let bound_this = closure.this;
                    call_flat(cx, callee, bound_this, function, &closure.fun, argc, function_call_kind)
                }
                _ => call_generic(cx, callee, this, argc, function_call_kind, call_ip),
            }
        } else {
            call_generic(cx, callee, this, argc, function_call_kind, call_ip)
        }
    }

    pub fn jmpfalsep(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack();

        let jump = !value.is_truthy(&mut cx.scope);

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpfalsenp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = !value.is_truthy(&mut cx.scope);

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmptruep(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack();

        let jump = value.is_truthy(&mut cx.scope);

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmptruenp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = value.is_truthy(&mut cx.scope);

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpnullishp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack();

        let jump = value.is_nullish();

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpnullishnp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = value.is_nullish();

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpundefinedp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.pop_stack();

        let jump = value.is_undefined();

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpundefinednp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;
        let value = cx.peek_stack();

        let jump = value.is_undefined();

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let offset = cx.fetchw_and_inc_ip() as i16;

        let ip = cx.frames.current_ip();
        cx.frames.set_ip(ip + offset);

        Ok(None)
    }

    pub fn loop_backjmp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let data = extract::<LoopBackjumpData>(&mut cx);
        let ip = cx.frames.current_ip();
        let target_ip = ip + data.offset;

        if unlikely(!data.hotness.is_disabled()) {
            // Slow path: we've either iterated less than 128 times, or this is the 128th time and we can try to optimize.
            let hotness = data.hotness.try_increment();

            match hotness {
                Some(hotness) => {
                    // Still counting.
                    cx.frames.set_byte(ip - 3, hotness.raw());
                }
                None => {
                    // We've saturated the counter. Attempt to JIT.

                    let func = jit::compile_loop_region(&mut cx.scope, target_ip, ip);
                    let result = func.call(&mut cx);
                    match result {
                        JitReturn::Normal { ip } => {
                            cx.frames.set_ip(ip);
                        }
                        JitReturn::Exception { value: _ } => todo!(),
                    }
                }
            }
        }

        cx.frames.set_ip(target_ip);

        Ok(None)
    }

    pub fn storelocal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = BackLocalId(cx.fetchw_and_inc_ip());
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let value = cx.get_local(id);
                let right = cx.pop_stack_rooted();
                let res = $op(value, right, &mut cx)?;
                cx.set_local(id, res.clone().into());
                cx.stack.push(res);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let value = cx.get_local(id);
                let value = Value::number(value.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(value, one, &mut cx)?;
                cx.set_local(id, res.clone().into());
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let value = cx.get_local(id);
                let value = Value::number(value.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(value, one, &mut cx)?;
                cx.set_local(id, res.into());
                cx.stack.push(value);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                // NOTE: Does not need to be rooted.
                let value = cx.pop_stack();
                cx.set_local(id, value);
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

    pub fn ldlocal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = BackLocalId(cx.fetchw_and_inc_ip());
        let value = cx.get_local(id);

        cx.stack.push(value);
        Ok(None)
    }

    fn with_arraylit_elements(
        cx: &mut DispatchContext<'_>,
        len: usize,
        stack_values: usize,
        mut fun: impl FnMut(ArrayElement),
    ) -> Result<(), Unrooted> {
        let mut iter = ForwardSequence::<ArrayElement>::from_len(cx, len, stack_values);
        while let Some(element) = iter.next_front(cx) {
            match element? {
                ArrayElement::Single(value) => fun(ArrayElement::Single(value)),
                ArrayElement::Spread(source, len) => {
                    for i in 0..len {
                        let value = source
                            .get_property(i.to_key(&mut cx.scope), &mut cx.scope)?
                            .root(&mut cx.scope);
                        fun(ArrayElement::Single(value));
                    }
                }
                ArrayElement::Hole(count) => fun(ArrayElement::Hole(count)),
            }
        }
        let truncate_to = cx.stack.len() - stack_values;
        cx.stack.truncate(truncate_to);

        debug_assert!(iter.next_front(cx).is_none());
        Ok(())
    }

    fn arraylit_holey(cx: &mut DispatchContext<'_>, len: usize, stack_values: usize) -> Result<Array, Unrooted> {
        let mut table = ArrayTable::new();
        with_arraylit_elements(cx, len, stack_values, |element| match element {
            ArrayElement::Single(value) => table.push(PropertyValue::static_default(value)),
            ArrayElement::Hole(hole) => table.resize(table.len() + hole),
            ArrayElement::Spread(..) => unreachable!(),
        })?;
        Ok(Array::from_table(&cx.scope, table))
    }

    fn arraylit_dense(cx: &mut DispatchContext<'_>, len: usize) -> Result<Array, Unrooted> {
        // Dense implies len == stack_values
        let mut new_elements = Vec::with_capacity(len);
        with_arraylit_elements(cx, len, len, |element| match element {
            ArrayElement::Single(value) => new_elements.push(PropertyValue::static_default(value)),
            ArrayElement::Spread(..) | ArrayElement::Hole(_) => unreachable!(),
        })?;
        Ok(Array::from_vec(new_elements, &cx.scope))
    }

    pub fn arraylit(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let len = cx.fetchw_and_inc_ip() as usize;
        let stack_values = cx.fetchw_and_inc_ip() as usize;
        // Split up into two functions as a non-holey array literal can be evaluated more efficiently
        let array = if len == stack_values {
            arraylit_dense(&mut cx, len)?
        } else {
            arraylit_holey(&mut cx, len, stack_values)?
        };

        let handle = cx.scope.register(array);
        cx.stack.push(Value::object(handle));
        Ok(None)
    }

    pub fn objlit(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let key_count = cx.fetchw_and_inc_ip() as usize;
        let stack_value_count = cx.fetchw_and_inc_ip() as usize;
        let mut iter = ForwardSequence::<ObjectProperty>::from_len(&mut cx, key_count, stack_value_count);

        let obj = OrdObject::new(&cx.scope);
        while let Some(property) = iter.next_front(&mut cx) {
            match property? {
                ObjectProperty::Static { key, value } => drop(obj.set_property(key, value, &mut cx.scope)),
                ObjectProperty::Getter { key, value } => match obj.get_own_property_descriptor(key, &mut cx.scope)? {
                    Some(prop) => {
                        obj.set_property(key, prop.with_getter(value), &mut cx.scope)?;
                    }
                    None => {
                        obj.set_property(key, PropertyValue::getter_default(value), &mut cx.scope)?;
                    }
                },
                ObjectProperty::Setter { key, value } => match obj.get_own_property_descriptor(key, &mut cx.scope)? {
                    Some(prop) => {
                        obj.set_property(key, prop.with_setter(value), &mut cx.scope)?;
                    }
                    None => {
                        obj.set_property(key, PropertyValue::setter_default(value), &mut cx.scope)?;
                    }
                },
                ObjectProperty::Spread(value) => {
                    if let ValueKind::Object(object) = value.unpack() {
                        for key in object.own_keys(&mut cx.scope, OwnKeysMode::OnlyEnumerable)? {
                            let key = PropertyKey::from_value(&mut cx.scope, key)?;
                            if let Some(value) = object.get_own_property_descriptor(key, &mut cx.scope)? {
                                obj.set_property(key, value, &mut cx.scope)?;
                            }
                        }
                    }
                }
            }
        }

        let stack_len = cx.stack.len();
        cx.stack.truncate(stack_len - stack_value_count);

        let handle = cx.scope.register(obj);
        cx.stack.push(handle.into());

        Ok(None)
    }

    pub fn assign_properties(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let key_count = cx.fetchw_and_inc_ip() as usize;
        let stack_value_count = cx.fetchw_and_inc_ip() as usize;

        let target = cx.pop_stack_rooted();
        let mut iter = ForwardSequence::<ObjectProperty>::from_len(&mut cx, key_count, stack_value_count);

        while let Some(property) = iter.next_front(&mut cx) {
            let property = property?;
            let is_getter = matches!(property, ObjectProperty::Getter { .. });

            match property {
                ObjectProperty::Static { key, value } => target.set_property(key, value, &mut cx.scope)?,
                ObjectProperty::Getter { key, value } | ObjectProperty::Setter { key, value } => {
                    let prop = target.get_property_descriptor(key, &mut cx.scope)?;
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

                    target.set_property(key, prop, &mut cx.scope)?;
                }
                ObjectProperty::Spread(_) => unimplemented!("spread operator in AssignProperties"),
            }
        }

        let stack_len = cx.stack.len();
        cx.stack.truncate(stack_len - stack_value_count);

        Ok(None)
    }

    pub fn staticpropertyaccess(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();

        let ident = JsString::from(cx.constants().symbols[SymbolConstant(id)]);

        let preserve_this = cx.fetch_and_inc_ip() == 1;

        let target = if preserve_this {
            cx.peek_stack().root(&mut cx.scope)
        } else {
            cx.pop_stack().root(&mut cx.scope)
        };

        let value = target.get_property(ident.to_key(&mut cx.scope), &mut cx.scope)?;
        cx.push_stack(value);
        Ok(None)
    }

    pub fn staticpropertyassign(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();
        let id = cx.fetchw_and_inc_ip();
        let key = JsString::from(cx.constants().symbols[SymbolConstant(id)]);

        macro_rules! op {
            ($op:expr) => {{
                let (target, value) = cx.pop_stack2_rooted();

                let p = target
                    .get_property(key.to_key(&mut cx.scope), &mut cx.scope)?
                    .root(&mut cx.scope);
                let res = $op(p, value, &mut cx)?;

                target.set_property(
                    key.to_key(&mut cx.scope),
                    PropertyValue::static_default(res.clone()),
                    &mut cx.scope,
                )?;
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let target = cx.pop_stack_rooted();
                let prop = target
                    .get_property(key.to_key(&mut cx.scope), &mut cx.scope)?
                    .root(&mut cx.scope);
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(prop, one, &mut cx)?;
                target.set_property(
                    key.to_key(&mut cx.scope),
                    PropertyValue::static_default(res),
                    &mut cx.scope,
                )?;
                cx.stack.push(prop);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let target = cx.pop_stack_rooted();
                let prop = target
                    .get_property(key.to_key(&mut cx.scope), &mut cx.scope)?
                    .root(&mut cx.scope);
                let prop = Value::number(prop.to_number(&mut cx)?);
                // TODO: check that it encodes at comptime, if not make a constant Value::ONE
                let one = Value::number(1.0);
                let res = $op(prop, one, &mut cx)?;
                target.set_property(
                    key.to_key(&mut cx.scope),
                    PropertyValue::static_default(res.clone()),
                    &mut cx.scope,
                )?;
                cx.stack.push(res);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let (target, value) = cx.pop_stack2_rooted();
                target.set_property(
                    key.to_key(&mut cx.scope),
                    PropertyValue::static_default(value),
                    &mut cx.scope,
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

    pub fn dynamicpropertyassign(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let (target, value, key) = cx.pop_stack3_rooted();

                let key = PropertyKey::from_value(&mut cx, key)?;
                let prop = target
                    .get_property(key.clone(), &mut cx.scope)?
                    .root(&mut cx.scope);

                let result = $op(prop, value, &mut cx)?;

                target.set_property(key, PropertyValue::static_default(result.clone()), &mut cx.scope)?;
                cx.stack.push(result);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let (target, key) = cx.pop_stack2_rooted();
                let key = PropertyKey::from_value(&mut cx, key)?;
                let prop = target
                    .get_property(key.clone(), &mut cx.scope)?
                    .root(&mut cx.scope);
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(prop, one, &mut cx)?;
                target.set_property(key, PropertyValue::static_default(res), &mut cx.scope)?;
                cx.stack.push(prop);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let (target, key) = cx.pop_stack2_rooted();
                let key = PropertyKey::from_value(&mut cx, key)?;
                let prop = target
                    .get_property(key.clone(), &mut cx.scope)?
                    .root(&mut cx.scope);
                let prop = Value::number(prop.to_number(&mut cx)?);
                let one = Value::number(1.0);
                let res = $op(prop, one, &mut cx)?;
                target.set_property(key, PropertyValue::static_default(res.clone()), &mut cx.scope)?;
                cx.stack.push(res);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let (target, value, key) = cx.pop_stack3_rooted();

                let key = PropertyKey::from_value(&mut cx, key)?;

                target.set_property(key, PropertyValue::static_default(value), &mut cx.scope)?;
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

    pub fn dynamicpropertyaccess(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let key = cx.pop_stack_rooted();

        let preserve_this = cx.fetch_and_inc_ip() == 1;

        let target = if preserve_this {
            cx.peek_stack().root(&mut cx.scope)
        } else {
            cx.pop_stack().root(&mut cx.scope)
        };

        let key = PropertyKey::from_value(&mut cx, key)?;

        let value = target.get_property(key, &mut cx.scope)?;
        cx.push_stack(value);
        Ok(None)
    }

    pub fn ldlocalext(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = ExternalId(cx.fetchw_and_inc_ip());
        let value = Value::external(cx.get_external(id).id());

        // Unbox external values such that any use will create a copy
        let value = value.unbox_external(&cx.scope);

        cx.stack.push(value);
        Ok(None)
    }

    fn assign_to_external(vm: &mut Vm, handle: ExternalValue, value: Value) {
        unsafe { ExternalValue::replace(vm, handle, value) };
    }

    pub fn storelocalext(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = ExternalId(cx.fetchw_and_inc_ip());
        let kind = AssignKind::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! op {
            ($op:expr) => {{
                let value = Value::external(cx.get_external(id).id()).unbox_external(&cx.scope);
                let right = cx.pop_stack_rooted();
                let res = $op(value, right, &mut cx)?;
                let external = cx.scope.get_external(id);
                assign_to_external(&mut cx.scope, external, res.clone());
                cx.stack.push(res);
            }};
        }

        macro_rules! prefix {
            ($op:expr) => {{
                let value = Value::external(cx.get_external(id).id()).unbox_external(&cx.scope);
                let right = Value::number(1.0);
                let res = $op(value, right, &mut cx)?;
                let external = cx.scope.get_external(id);
                assign_to_external(&mut cx.scope, external, res.clone());
                cx.stack.push(res);
            }};
        }

        macro_rules! postfix {
            ($op:expr) => {{
                let value = Value::external(cx.get_external(id).id()).unbox_external(&cx.scope);
                let right = Value::number(1.0);
                let res = $op(value, right, &mut cx)?;
                let external = cx.scope.get_external(id);
                assign_to_external(&mut cx.scope, external, res);
                cx.stack.push(value);
            }};
        }

        match kind {
            AssignKind::Assignment => {
                let value = cx.pop_stack_rooted();
                let external = cx.scope.get_external(id);
                assign_to_external(&mut cx.scope, external, value);
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

    pub fn try_block(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let mut compute_dist_ip = || {
            let distance = extract::<Option<u16>>(&mut cx)?;
            let ip = cx.frames.current_ip();
            Some(ip + distance as u32)
        };

        let catch_ip = compute_dist_ip();
        let finally_ip = compute_dist_ip();
        let frame_idx = cx.frames.current_id();

        cx.try_blocks.push(TryBlock {
            catch_ip,
            finally_ip,
            frame_idx,
        });

        Ok(None)
    }

    pub fn pop_try(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.try_blocks.pop();
        Ok(None)
    }

    pub fn throw(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        Err(cx.pop_stack())
    }

    pub fn type_of(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let ty = value.type_of(&cx.scope).as_value();
        cx.stack.push(ty);
        Ok(None)
    }

    pub fn type_of_ident(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let id = cx.fetchw_and_inc_ip();
        let ident = JsString::from(cx.constants().symbols[SymbolConstant(id)]);
        let prop = cx
            .global
            .get_property(ident.to_key(&mut cx.scope), &mut cx.scope)?
            .root(&mut cx.scope);

        let ty = prop.type_of(&cx.scope).as_value();
        cx.stack.push(ty);
        Ok(None)
    }

    pub fn yield_(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack();
        Ok(Some(HandleResult::Yield(value)))
    }

    pub fn await_(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack();
        Ok(Some(HandleResult::Await(value)))
    }

    pub fn import_dyn(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();

        let _ret = match cx.params.dynamic_import_callback {
            Some(cb) => cb(&mut cx, value)?,
            None => throw!(cx, Error, "Dynamic imports are disabled for this context"),
        };

        // TODO: dynamic imports are currently statements, making them useless
        // TODO: make them an expression and push ret on stack

        Ok(None)
    }

    pub fn import_static(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ty = StaticImportKind::from_repr(cx.fetch_and_inc_ip()).expect("Invalid import kind");
        let local_id = BackLocalId(cx.fetchw_and_inc_ip());
        let path_id = cx.fetchw_and_inc_ip();

        let path = cx.constants().symbols[SymbolConstant(path_id)];

        let value = match cx.params.static_import_callback {
            Some(cb) => cb(&mut cx, ty, path.into())?,
            None => throw!(cx, Error, "Static imports are disabled for this context."),
        };

        cx.set_local(local_id, value);

        Ok(None)
    }

    pub fn export_default(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        // NOTE: Does not need to be rooted. Storing it in frame state counts as being rooted.
        let value = cx.pop_stack();

        match cx.frames.current_state_mut() {
            FrameState::Module(module) => {
                module.default = Some(value);
            }
            _ => throw!(cx, Error, "Export is only available at the top level in modules"),
        }

        Ok(None)
    }

    pub fn export_named(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let mut iter = BackwardSequence::<ExportProperty>::new_u16(&mut cx);
        while let Some(prop) = iter.next(&mut cx) {
            let ExportProperty(value, ident) = prop?;

            match cx.frames.current_state_mut() {
                FrameState::Module(exports) => exports.named.push((ident, value)),
                _ => throw!(cx, Error, "Export is only available at the top level in modules"),
            }
        }
        Ok(None)
    }

    pub fn debugger(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        if let Some(cb) = cx.params().debugger_callback {
            cb(&mut cx)?;
        }

        Ok(None)
    }

    pub fn this(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.frames.current_this().to_value(&mut cx.scope)?;
        cx.stack.push(value);
        Ok(None)
    }

    pub fn bindthis(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        cx.frames.set_this(This::bound(value));
        Ok(None)
    }

    pub fn global_this(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let global = cx.global;
        cx.stack.push(Value::object(global));
        Ok(None)
    }

    pub fn super_(cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        throw!(cx, SyntaxError, "`super` keyword unexpected in this context");
    }

    pub fn undef(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::undefined());
        Ok(None)
    }

    pub fn infinity(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::number(f64::INFINITY));
        Ok(None)
    }

    pub fn nan(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        cx.stack.push(Value::number(f64::NAN));
        Ok(None)
    }

    pub fn call_symbol_iterator(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();
        let symbol_iterator = cx.statics.symbol_iterator;
        let iterable = value
            .get_property(symbol_iterator.to_key(&mut cx.scope), &mut cx.scope)?
            .root(&mut cx.scope);
        let iterator = iterable.apply(This::bound(value), CallArgs::empty(), &mut cx.scope)?;
        cx.push_stack(iterator);
        Ok(None)
    }

    pub fn call_for_in_iterator(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let value = cx.pop_stack_rooted();

        let keys = match value.unpack() {
            ValueKind::Object(obj) => obj.own_keys(&mut cx.scope, OwnKeysMode::OnlyEnumerable)?,
            ValueKind::External(obj) => obj.own_keys(&mut cx.scope, OwnKeysMode::OnlyEnumerable)?,
            _ => Vec::new(),
        }
        .into_iter()
        .map(PropertyValue::static_default)
        .collect();

        let keys = Array::from_vec(keys, &cx.scope);
        let keys = cx.register(keys);
        let iter = ArrayIterator::new(&mut cx, Value::object(keys))?;
        let iter = cx.register(iter);
        cx.stack.push(Value::object(iter));
        Ok(None)
    }

    pub fn delete_property_dynamic(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let (property, target) = cx.pop_stack2_rooted();
        let key = PropertyKey::from_value(&mut cx, property)?;
        let value = target.delete_property(key, &mut cx.scope)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value.root(&mut cx.scope).unpack(), ValueKind::Undefined(..));
        cx.stack.push(Value::boolean(did_delete));
        Ok(None)
    }

    pub fn delete_property_static(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let target = cx.pop_stack_rooted();
        let cid = cx.fetchw_and_inc_ip();
        let con = JsString::from(cx.constants().symbols[SymbolConstant(cid)]);
        let value = target.delete_property(con.to_key(&mut cx.scope), &mut cx.scope)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value.root(&mut cx.scope).unpack(), ValueKind::Undefined(..));
        cx.stack.push(Value::boolean(did_delete));
        Ok(None)
    }

    pub fn objdestruct(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let rest_id = match cx.fetchw_and_inc_ip() {
            u16::MAX => None,
            n => Some(BackLocalId(n)),
        };
        let obj = cx.pop_stack_rooted();

        let mut idents = Vec::new();

        let mut iter = BackwardSequence::<(bool, NumberWConstant, IdentW)>::new_u16(&mut cx);
        while let Some((has_default, NumberWConstant(id), IdentW(ident))) = iter.next_infallible(&mut cx) {
            if rest_id.is_some() {
                idents.push(ident);
            }

            let mut prop = obj
                .get_property(ident.to_key(&mut cx.scope), &mut cx.scope)?
                .root(&mut cx.scope);
            if has_default {
                // NB: we need to at least pop it from the stack even if the property exists
                let default = cx.pop_stack_rooted();
                if matches!(prop.unpack(), ValueKind::Undefined(_)) {
                    prop = default;
                }
            }
            cx.set_local(BackLocalId(id as u16), prop.into());
        }

        if let Some(rest_id) = rest_id {
            let keys = obj
                .own_keys(&mut cx.scope, OwnKeysMode::OnlyEnumerable)?
                .into_iter()
                .filter_map(|s| match s.unpack() {
                    ValueKind::String(s) => (!idents.contains(&s)).then_some(s),
                    _ => unreachable!("own_keys returned non-string"),
                })
                .collect::<Vec<_>>();

            let rest = OrdObject::new(&cx.scope);
            let rest = cx.scope.register(rest);
            for key in keys {
                let value = obj
                    .get_property(key.to_key(&mut cx.scope), &mut cx.scope)?
                    .root(&mut cx.scope);
                rest.set_property(
                    key.to_key(&mut cx.scope),
                    PropertyValue::static_default(value),
                    &mut cx.scope,
                )?;
            }

            cx.set_local(rest_id.into(), Value::object(rest).into());
        }

        Ok(None)
    }

    pub fn arraydestruct(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let array = cx.pop_stack_rooted();

        let mut iter = BackwardSequence::<Option<(bool, NumberWConstant)>>::new_u16(&mut cx).enumerate();

        while let Some((i, id)) = iter.next_infallible(&mut cx) {
            if let Some((has_default, NumberWConstant(id))) = id {
                let id = BackLocalId(id as u16);
                let mut prop = array
                    .get_property(i.to_key(&mut cx.scope), &mut cx.scope)?
                    .root(&mut cx.scope);

                if has_default {
                    // NB: we need to at least pop it from the stack even if the property exists
                    let default = cx.pop_stack_rooted();
                    if matches!(prop.unpack(), ValueKind::Undefined(_)) {
                        prop = default;
                    }
                }
                cx.set_local(id, prop.into());
            }
        }

        Ok(None)
    }

    pub fn intrinsic_op(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let op = IntrinsicOperation::from_repr(cx.fetch_and_inc_ip()).unwrap();

        macro_rules! lr_as_num_spec {
            () => {{
                // Unrooted is technically fine here, nothing can trigger a GC cycle
                // OK to remove if it turns out to be a useful opt
                // TODO: this can be optimized by reinterpreting it as a number directly, but could be potentially quite unsafe
                let (left, right) = cx.pop_stack2_rooted();
                match (left.unpack(), right.unpack()) {
                    (ValueKind::Number(l), ValueKind::Number(r)) => (l.0, r.0),
                    _ => unreachable!(),
                }
            }};
        }

        macro_rules! bin_op {
            ($fun:expr) => {{
                let (l, r) = lr_as_num_spec!();
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
                cx.stack.push(Value::boolean(l $op r));
            }};
        }

        macro_rules! postfix {
            ($op:tt) => {{
                let id = BackLocalId(cx.fetch_and_inc_ip() as u16);
                let local = match cx.get_local(id).unpack() {
                    ValueKind::Number(n) => n,
                    _ => unreachable!(),
                };
                cx.set_local(id, Value::number(local.0 $op 1.0).into());
                cx.stack.push(Value::number(local.0));
            }};
        }

        macro_rules! prefix {
            ($op:tt) => {{
                let id = BackLocalId(cx.fetch_and_inc_ip() as u16);
                let local = match cx.get_local(id).unpack() {
                    ValueKind::Number(n) => n,
                    _ => unreachable!(),
                };
                let new = Value::number(local.0 $op 1.0);
                cx.set_local(id, new.into());
                cx.stack.push(new);
            }};
        }

        macro_rules! bin_op_numl_constr {
            ($op:tt) => {{
                let left = match cx.pop_stack_rooted().unpack() {
                    ValueKind::Number(n) => n.0,
                    _ => unreachable!(),
                };
                let right = cx.fetch_and_inc_ip() as f64;
                cx.stack.push(Value::boolean(left $op right));
            }};
        }

        fn logical_op_numl_u32r_n<F: FnOnce(f64, f64) -> bool>(mut cx: DispatchContext<'_>, f: F) {
            let vm: &mut Vm = &mut cx;

            let Some(value) = vm.stack.last_mut() else {
                unreachable!()
            };
            let ValueKind::Number(Number(left)) = value.unpack() else {
                unreachable!()
            };
            let right = vm.frames.fetch32_and_inc_ip() as f64;

            *value = Value::boolean(f(left, right));
        }

        macro_rules! fn_call {
            ($fun:ident, $k:expr, $v:expr) => {{
                let argc = cx.fetch_and_inc_ip();
                let args = cx.drain_stack_rooted(argc.into()).collect::<CallArgs>();
                let fun = cx.statics.$fun.clone();

                if unlikely(!cx.builtins_purity()) {
                    for arg in &args {
                        cx.scope.add(arg.clone());
                    }

                    // Builtins impure, fallback to slow dynamic property lookup
                    let k = cx
                        .global
                        .clone()
                        .get_property($k.to_key(&mut cx.scope), &mut cx.scope)?
                        .root(&mut cx.scope);
                    let fun = k
                        .get_property($v.to_key(&mut cx.scope), &mut cx.scope)?
                        .root(&mut cx.scope);
                    let result = fun.apply(This::default(), args, &mut cx.scope)?;
                    cx.push_stack(result);
                } else {
                    // Fastpath: call builtin directly
                    // TODO: should we add to externals?
                    let result = fun.apply(This::default(), args, &mut cx.scope)?;
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

    pub fn new_target(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        if let FrameState::Function {
            new_target: Some(new_target),
            ..
        } = *cx.frames.current_state()
        {
            cx.stack.push(Value::object(new_target));
        } else {
            cx.stack.push(Value::undefined());
        }
        Ok(None)
    }

    pub fn nop(_: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        Ok(None)
    }
}

macro_rules! define_instruction_lut {
    (
        $($variant:path => $handler:ident),*
    ) => {
        type HandlerFn = fn(DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted>;
        pub static INSTRUCTION_LUT: [HandlerFn; 256] = {
            let mut lut: [HandlerFn; 256] = [handlers::nop as HandlerFn; 256];
            let mut i = 0;
            $(
                debug_assert!(i == $variant as usize);
                debug_assert!(i < 256);
                lut[i] = handlers::$handler;
                i += 1;
            )*
            lut
        };
    };
}

define_instruction_lut! {
    Instruction::Add => add,
    Instruction::Sub => sub,
    Instruction::Mul => mul,
    Instruction::Div => div,
    Instruction::Rem => rem,
    Instruction::Pow => pow,
    Instruction::Gt => gt,
    Instruction::Ge => ge,
    Instruction::Lt => lt,
    Instruction::Le => le,
    Instruction::Eq => eq,
    Instruction::Ne => ne,
    Instruction::Pop => pop,
    Instruction::LdLocal => ldlocal,
    Instruction::LdGlobal => ldglobal,
    Instruction::String => string_constant,
    Instruction::Boolean => boolean_constant,
    Instruction::Number => number_constant,
    Instruction::Regex => regex_constant,
    Instruction::Null => null_constant,
    Instruction::Undefined => undefined_constant,
    Instruction::Function => function_constant,
    Instruction::Pos => pos,
    Instruction::Neg => neg,
    Instruction::TypeOf => type_of,
    Instruction::TypeOfGlobalIdent => type_of_ident,
    Instruction::BitNot => bitnot,
    Instruction::Not => not,
    Instruction::StoreLocal => storelocal,
    Instruction::StoreGlobal => storeglobal,
    Instruction::Ret => ret,
    Instruction::Call => call,
    Instruction::JmpFalseP => jmpfalsep,
    Instruction::Jmp => jmp,
    Instruction::LoopBackJmp => loop_backjmp,
    Instruction::StaticPropAccess => staticpropertyaccess,
    Instruction::DynamicPropAccess => dynamicpropertyaccess,
    Instruction::ArrayLit => arraylit,
    Instruction::ObjLit => objlit,
    Instruction::BindThis => bindthis,
    Instruction::This => this,
    Instruction::StaticPropAssign => staticpropertyassign,
    Instruction::DynamicPropAssign => dynamicpropertyassign,
    Instruction::LdLocalExt => ldlocalext,
    Instruction::StoreLocalExt => storelocalext,
    Instruction::StrictEq => strict_eq,
    Instruction::StrictNe => strict_ne,
    Instruction::Try => try_block,
    Instruction::PopTry => pop_try,
    Instruction::FinallyEnd => finally_end,
    Instruction::Throw => throw,
    Instruction::Yield => yield_,
    Instruction::JmpFalseNP => jmpfalsenp,
    Instruction::JmpTrueP => jmptruep,
    Instruction::JmpTrueNP => jmptruenp,
    Instruction::JmpNullishP => jmpnullishp,
    Instruction::JmpNullishNP => jmpnullishnp,
    Instruction::JmpUndefinedNP => jmpundefinednp,
    Instruction::JmpUndefinedP => jmpundefinedp,
    Instruction::BitOr => bitor,
    Instruction::BitXor => bitxor,
    Instruction::BitAnd => bitand,
    Instruction::BitShl => bitshl,
    Instruction::BitShr => bitshr,
    Instruction::BitUshr => bitushr,
    Instruction::ObjIn => objin,
    Instruction::InstanceOf => instanceof,
    Instruction::ImportDyn => import_dyn,
    Instruction::ImportStatic => import_static,
    Instruction::ExportDefault => export_default,
    Instruction::ExportNamed => export_named,
    Instruction::Debugger => debugger,
    Instruction::Global => global_this,
    Instruction::Super => super_,
    Instruction::Undef => undef,
    Instruction::Await => await_,
    Instruction::Nan => nan,
    Instruction::Infinity => infinity,
    Instruction::IntrinsicOp => intrinsic_op,
    Instruction::CallSymbolIterator => call_symbol_iterator,
    Instruction::CallForInIterator => call_for_in_iterator,
    Instruction::DeletePropertyStatic => delete_property_static,
    Instruction::DeletePropertyDynamic => delete_property_dynamic,
    Instruction::ObjDestruct => objdestruct,
    Instruction::ArrayDestruct => arraydestruct,
    Instruction::AssignProperties => assign_properties,
    Instruction::DelayedReturn => delayed_ret,
    Instruction::NewTarget => new_target,
    Instruction::Nop => nop
}

pub fn handle(vm: &mut Vm, instruction: Instruction) -> Result<Option<HandleResult>, Unrooted> {
    let cx = DispatchContext::new(vm.scope());
    INSTRUCTION_LUT[instruction as usize](cx)
}
