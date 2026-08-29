use dash_middle::compiler::constant::ConstantPool;
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

    pub fn peek_stack(&self) -> Unrooted {
        Unrooted::new(
            *self
                .stack
                .last()
                .expect("Bytecode attempted to peek stack value, but nothing was on the stack"),
        )
    }

    pub fn peek_stack_rooted(&mut self) -> Value {
        self.peek_stack().root(&mut self.scope)
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

mod handlers {
    use dash_middle::compiler::FunctionCallKind;
    use dash_middle::compiler::constant::FunctionConstant;
    use dash_middle::compiler::external::{External, PossiblyExternalId};
    use dash_middle::compiler::extract::{BackwardSequence, ForwardSequence, extract_back_infallible};
    use dash_middle::compiler::operands::{
        AddOperands, ArrayDestructuringMember, ArrayDestructuringOperands, ArrayLiteralElement, ArrayLiteralOperands,
        AssignKind, AssignPropertiesOperands, AwaitOperands, BinaryOperator, BindThisOperands, BitandOperands,
        BitnotOperands, BitorOperands, BitshlOperands, BitshrOperands, BitushrOperands, BitxorOperands,
        BooleanConstantOperands, BooleanConstantWide, CallOperands, CallSymbolIteratorOperands,
        ConditionalJumpNoPopOperands, ConditionalJumpPopOperands, DelayedRetOperands, DeletePropertyDynamicOperands,
        DeletePropertyStaticOperands, DivOperands, DynamicPropertyAccessOperands, DynamicPropertyAssignOperands,
        EqOperands, ExportDefaultOperands, ExportNamedOperands, ExportProperty, FinallyEndOperands,
        ForInIteratorOperands, FunctionConstantOperands, GeOperands, GtOperands, ImportDynOperands,
        ImportStaticOperands, InstanceofOperands, IntrinsicCallOperands, IntrinsicKind, IntrinsicOperands,
        JmpFalseNoPopOperands, JmpFalsePopOperands, JmpNullishNoPopOperands, JmpNullishPopOperands, JmpOperands,
        JmpTrueNoPopOperands, JmpTruePopOperands, JmpUndefinedNoPopOperands, JmpUndefinedPopOperands, LdGlobalOperands,
        LdLocalExtOperands, LdLocalOperands, LeOperands, LtOperands, MulOperands, NeOperands, NegOperands, NotOperands,
        NumberConstantOperands, NumberConstantWide, NumberInline8, NumberInline32, ObjectDestructuringMember,
        ObjectDestructuringOperands, ObjectInOperands, ObjectLiteralOperands, ObjectProperty, OptionDiscriminatedByte,
        OptionNoneMax, PopOperands, PosOperands, PowOperands, RegexConstantOperands, RemOperands, RetOperands,
        StaticPropertyAccessOperands, StaticPropertyAssignOperands, StoreGlobalOperands, StoreLocalExtOperands,
        StoreLocalOperands, StrictEqOperands, StrictNeOperands, StringConstantOperands, SubOperands,
        SymbolConstantWide, ThrowOperands, TryCatchDepth, TypeofIdentOperands, TypeofOperands, YieldOperands,
    };
    use dash_middle::interner::{Symbol, sym};
    use dash_middle::iterator_with::{self, InfallibleIteratorWith, IteratorWith};
    use dash_middle::parser::statement::{Asyncness, FunctionKind as ParserFunctionKind};
    use if_chain::if_chain;
    use smallvec::SmallVec;
    use std::convert::Infallible;
    use std::ops::{Add, ControlFlow, Div, Mul, Rem, Sub};
    use std::rc::Rc;

    use crate::frame::{FrameState, Ip, Sp, TryBlock};
    use crate::gc::ObjectId;
    use crate::throw;
    use crate::util::likely;
    use crate::value::array::table::ArrayTable;
    use crate::value::array::{Array, ArrayIterator};
    use crate::value::function::args::CallArgs;
    use crate::value::function::r#async::AsyncFunction;
    use crate::value::function::closure::Closure;
    use crate::value::function::generator::GeneratorFunction;
    use crate::value::function::user::UserFunction;
    use crate::value::function::{Function, FunctionKind, adjust_stack_from_flat_call, this_for_new_target};
    use crate::value::object::{Object, OrdObject, OwnKeysMode, PropertyValue, This, ThisKind};
    use crate::value::ops::conversions::ValueConversion;
    use crate::value::ops::equality;
    use crate::value::propertykey::{PropertyKey, ToPropertyKey};
    use crate::value::regex::RegExp;
    use crate::value::{Unpack, ValueKind};

    use super::*;

    impl dash_middle::compiler::extract::ExtractFront<DispatchContext<'_>> for Value {
        type Exception = Infallible;

        fn extract_front<U>(
            cx: &mut DispatchContext<'_>,
            seq: &mut ForwardSequence<U>,
        ) -> Result<Self, Self::Exception> {
            let index = seq.next_stack_index();
            let value = cx.stack[index];
            cx.scope.add(value);
            Ok(value)
        }
    }

    impl dash_middle::compiler::extract::ExtractBack<DispatchContext<'_>> for Value {
        type Exception = Infallible;

        fn extract_back(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(cx.pop_stack_rooted())
        }
    }

    impl dash_middle::compiler::extract::ExtractBack<DispatchContext<'_>> for Unrooted {
        type Exception = Infallible;

        fn extract_back(cx: &mut DispatchContext<'_>) -> Result<Self, Self::Exception> {
            Ok(cx.pop_stack())
        }
    }

    impl<'vm> dash_middle::compiler::extract::ExtractSource for DispatchContext<'vm> {
        type Value = Value;
        type Unrooted = Unrooted;

        fn fetch_bytes<const N: usize>(&mut self) -> [u8; N] {
            self.frames.fetch_n_and_inc_ip()
        }

        fn constants(&self) -> &ConstantPool {
            self.constants()
        }

        fn pop_stack_rooted(&mut self) -> Self::Value {
            self.pop_stack_rooted()
        }

        fn pop_stack(&mut self) -> Self::Unrooted {
            self.pop_stack()
        }

        fn peek_stack(&self) -> Self::Unrooted {
            self.peek_stack()
        }

        fn stack_len(&self) -> usize {
            self.stack.len()
        }

        fn peek_stack_rooted(&mut self) -> Self::Value {
            self.peek_stack_rooted()
        }

        fn truncate_stack(&mut self, len: usize) {
            self.stack.truncate(len);
        }
    }

    pub fn string_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let StringConstantOperands(SymbolConstantWide(sym)) = extract_back_infallible(&mut cx);
        cx.push_stack(Value::string(sym.into()).into());
        Ok(None)
    }

    pub fn boolean_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let BooleanConstantOperands(BooleanConstantWide(value)) = extract_back_infallible(&mut cx);
        cx.push_stack(Value::boolean(value).into());
        Ok(None)
    }

    pub fn number_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let NumberConstantOperands(NumberConstantWide(value)) = extract_back_infallible(&mut cx);
        cx.push_stack(Value::number(value).into());
        Ok(None)
    }

    pub fn regex_constant(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let (regex, source) = extract_back_infallible::<_, RegexConstantOperands>(&mut cx).regex(&mut cx);

        let regex = RegExp::new(regex.clone(), JsString::from(source), &cx.scope);
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

        // let id = cx.fetchw_and_inc_ip();
        let FunctionConstantOperands(FunctionConstant(id)) = extract_back_infallible(&mut cx);
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
        let AddOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.add(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn sub(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let SubOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.sub(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn mul(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let MulOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.mul(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn div(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let DivOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.div(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn rem(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let RemOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.rem(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn pow(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let PowOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.pow(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn bitor(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let BitorOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.bitor(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn bitxor(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let BitxorOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.bitxor(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn bitand(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let BitandOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.bitand(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn bitshl(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let BitshlOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.bitshl(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn bitshr(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let BitshrOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.bitshr(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn bitushr(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let BitushrOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = left.bitushr(right, &mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn bitnot(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let BitnotOperands(value) = extract_back_infallible(&mut cx);
        let result = value.bitnot(&mut cx)?;
        cx.scope.stack.push(result);
        Ok(None)
    }

    pub fn objin(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ObjectInOperands { key, target } = extract_back_infallible(&mut cx);

        let key = key.to_js_string(&mut cx)?;
        let found = target
            .for_each_prototype(&mut cx, |sc, target| {
                let contains = target
                    .own_keys(sc, OwnKeysMode::All)?
                    .iter()
                    .any(|v| matches!(v.unpack(), ValueKind::String(s) if s == key));

                if contains {
                    Ok(ControlFlow::Break(()))
                } else {
                    Ok(ControlFlow::Continue(()))
                }
            })?
            .is_break();

        cx.stack.push(Value::boolean(found));
        Ok(None)
    }

    pub fn instanceof(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let InstanceofOperands { constructor, value } = extract_back_infallible(&mut cx);

        let is_instanceof = value.instanceof(&constructor, &mut cx).map(Value::boolean)?;
        cx.stack.push(is_instanceof);
        Ok(None)
    }

    pub fn lt(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let LtOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = equality::lt(left, right, &mut cx)?.into();
        cx.stack.push(result);
        Ok(None)
    }

    pub fn le(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let LeOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = equality::le(left, right, &mut cx)?.into();
        cx.stack.push(result);
        Ok(None)
    }

    pub fn gt(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let GtOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = equality::gt(left, right, &mut cx)?.into();
        cx.stack.push(result);
        Ok(None)
    }

    pub fn ge(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let GeOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = equality::ge(left, right, &mut cx)?.into();
        cx.stack.push(result);
        Ok(None)
    }

    pub fn eq(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let EqOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = equality::eq(left, right, &mut cx)?.into();
        cx.stack.push(result);
        Ok(None)
    }

    pub fn ne(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let NeOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = equality::ne(left, right, &mut cx)?.into();
        cx.stack.push(result);
        Ok(None)
    }

    pub fn strict_eq(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let StrictEqOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = Value::boolean(equality::strict_eq(left, right));
        cx.stack.push(result);
        Ok(None)
    }

    pub fn strict_ne(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let StrictNeOperands(BinaryOperator { left, right }) = extract_back_infallible(&mut cx);
        let result = Value::boolean(equality::strict_ne(left, right));
        cx.stack.push(result);
        Ok(None)
    }

    pub fn neg(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let NegOperands(value) = extract_back_infallible(&mut cx);
        let result = value.to_number(&mut cx)?;
        cx.stack.push(Value::number(-result));
        Ok(None)
    }

    pub fn pos(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let PosOperands(value) = extract_back_infallible(&mut cx);
        let result = value.to_number(&mut cx)?;
        cx.stack.push(Value::number(result));
        Ok(None)
    }

    pub fn not(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let NotOperands(value) = extract_back_infallible(&mut cx);
        let result = value.not(&mut cx.scope);
        cx.stack.push(result);
        Ok(None)
    }

    pub fn pop(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let PopOperands(_) = extract_back_infallible(&mut cx);
        Ok(None)
    }

    pub fn delayed_ret(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let DelayedRetOperands(value) = extract_back_infallible(&mut cx);
        cx.frames.set_delayed_ret(Some(Ok(value)));
        Ok(None)
    }

    pub fn finally_end(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let FinallyEndOperands(TryCatchDepth(tc_depth)) = extract_back_infallible(&mut cx);

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
        let RetOperands { tc_depth, value } = extract_back_infallible(&mut cx);
        let this = cx.pop_frame();
        ret_inner(cx, tc_depth.0, value, this)
    }

    pub fn ldglobal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let LdGlobalOperands(SymbolConstantWide(name)) = extract_back_infallible(&mut cx);

        let value = match cx.global.clone().extract::<OrdObject>(&cx.scope) {
            Some(value) => match value.get_own_property_descriptor(name.to_key(&mut cx.scope), &mut cx.scope)? {
                Some(value) => value.kind().get_or_apply(&mut cx, This::default())?,
                None => {
                    let name = cx.scope.interner.resolve(name).to_owned();
                    throw!(&mut cx, ReferenceError, "{} is not defined", name)
                }
            },
            None => cx.global.get_property(name.to_key(&mut cx.scope), &mut cx.scope)?,
        };

        cx.push_stack(value);
        Ok(None)
    }

    pub fn storeglobal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        fn binop(
            cx: &mut DispatchContext<'_>,
            name: Symbol,
            right: Value,
            op: impl FnOnce(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let left = cx
                .global
                .clone()
                .get_property(name.to_key(&mut cx.scope), &mut cx.scope)
                .root(&mut cx.scope)?;
            let res = op(left, right, &mut cx.scope)?;
            cx.global.clone().set_property(
                name.to_key(&mut cx.scope),
                PropertyValue::static_default(res.clone()),
                &mut cx.scope,
            )?;
            cx.stack.push(res);

            Ok(())
        }

        fn prefix(
            cx: &mut DispatchContext<'_>,
            name: Symbol,
            op: impl FnOnce(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let left = cx
                .global
                .clone()
                .get_property(name.to_key(&mut cx.scope), &mut cx.scope)
                .root(&mut cx.scope)?;
            let left = Value::number(left.to_number(&mut cx.scope)?);

            let right = Value::number(1.0);
            let res = op(left, right, &mut cx.scope)?;
            cx.global.clone().set_property(
                name.to_key(&mut cx.scope),
                PropertyValue::static_default(res),
                &mut cx.scope,
            )?;
            cx.stack.push(res);

            Ok(())
        }

        fn postfix(
            cx: &mut DispatchContext<'_>,
            name: Symbol,
            op: impl FnOnce(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let left = cx
                .global
                .clone()
                .get_property(name.to_key(&mut cx.scope), &mut cx.scope)
                .root(&mut cx.scope)?;
            let left = Value::number(left.to_number(&mut cx.scope)?);

            let right = Value::number(1.0);
            let res = op(left, right, &mut cx.scope)?;
            cx.global.clone().set_property(
                name.to_key(&mut cx.scope),
                PropertyValue::static_default(res),
                &mut cx.scope,
            )?;
            cx.stack.push(left);

            Ok(())
        }

        let StoreGlobalOperands {
            name: SymbolConstantWide(name),
            kind,
        } = extract_back_infallible(&mut cx);

        match kind {
            AssignKind::Assignment(value) => {
                let value = value.root(&mut cx.scope);
                cx.global.clone().set_property(
                    name.to_key(&mut cx.scope),
                    PropertyValue::static_default(value),
                    &mut cx.scope,
                )?;
                cx.stack.push(value);
            }
            AssignKind::AddAssignment(value) => binop(&mut cx, name, value, Value::add)?,
            AssignKind::SubAssignment(value) => binop(&mut cx, name, value, Value::sub)?,
            AssignKind::MulAssignment(value) => binop(&mut cx, name, value, Value::mul)?,
            AssignKind::DivAssignment(value) => binop(&mut cx, name, value, Value::div)?,
            AssignKind::RemAssignment(value) => binop(&mut cx, name, value, Value::rem)?,
            AssignKind::PowAssignment(value) => binop(&mut cx, name, value, Value::pow)?,
            AssignKind::ShlAssignment(value) => binop(&mut cx, name, value, Value::bitshl)?,
            AssignKind::ShrAssignment(value) => binop(&mut cx, name, value, Value::bitshr)?,
            AssignKind::UshrAssignment(value) => binop(&mut cx, name, value, Value::bitushr)?,
            AssignKind::BitAndAssignment(value) => binop(&mut cx, name, value, Value::bitand)?,
            AssignKind::BitOrAssignment(value) => binop(&mut cx, name, value, Value::bitor)?,
            AssignKind::BitXorAssignment(value) => binop(&mut cx, name, value, Value::bitxor)?,
            AssignKind::PrefixIncrement => prefix(&mut cx, name, Value::add)?,
            AssignKind::PrefixDecrement => prefix(&mut cx, name, Value::sub)?,
            AssignKind::PostfixIncrement => postfix(&mut cx, name, Value::add)?,
            AssignKind::PostfixDecrement => postfix(&mut cx, name, Value::sub)?,
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
        mut spread_indices: BackwardSequence<u8>,
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

        // If we have spread args, we need to "splice" values from iterables in.
        // This is hopefully rather "rare" (compared to regular call arguments),
        // so we can afford to do more work here in order to keep the common path fast.
        if spread_indices.remaining_len() > 0 {
            let mut spread_count = 0;

            let mut splice_args = Vec::new();
            while let Some(spread_index) = spread_indices.next_infallible(&mut cx) {
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
        spread_indices: BackwardSequence<u8>,
    ) -> Result<Option<HandleResult>, Unrooted> {
        let args = {
            let mut args = SmallVec::with_capacity(argc);

            let raw_args = cx.drain_stack_rooted(argc);

            if spread_indices.remaining_len() == 0 {
                // Fast path for no spread arguments
                args.extend(raw_args);
            } else {
                let mut indices_iter = <_ as IteratorWith<&mut DispatchContext<'_>>>::peekable(spread_indices);
                let raw_args = raw_args.collect::<SmallVec<[Value; 3]>>();

                for (index, value) in raw_args.into_iter().enumerate() {
                    let is_spread_argument = iterator_with::next_if(&mut indices_iter, &mut cx, |v| {
                        let v = *v;
                        let Ok(v) = v;
                        usize::from(v) == index
                    })
                    .is_some();

                    if is_spread_argument {
                        // Spread the value into the argument list
                        let len = value.length_of_array_like(&mut cx.scope)?;
                        for i in 0..len {
                            let value = value
                                .get_property(i.to_key(&mut cx.scope), &mut cx.scope)?
                                .root(&mut cx.scope);
                            args.push(value);
                        }
                    } else {
                        // Single value
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

        let CallOperands {
            argc,
            function_call_kind,
            has_this,
            spread_indices,
        } = extract_back_infallible(&mut cx);
        let argc = usize::from(argc);

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
                FunctionKind::User(user) => call_flat(
                    cx,
                    callee,
                    this,
                    function,
                    user,
                    argc,
                    function_call_kind,
                    spread_indices,
                ),
                FunctionKind::Closure(closure) => {
                    if function_call_kind == FunctionCallKind::Constructor {
                        throw!(cx.scope, TypeError, "closure cannot be called as a constructor")
                    }

                    call_flat(
                        cx,
                        callee,
                        closure.this,
                        function,
                        &closure.fun,
                        argc,
                        function_call_kind,
                        spread_indices,
                    )
                }
                _ => call_generic(cx, callee, this, argc, function_call_kind, call_ip, spread_indices),
            }
        } else {
            call_generic(cx, callee, this, argc, function_call_kind, call_ip, spread_indices)
        }
    }

    pub fn jmpfalsep(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpFalsePopOperands(ConditionalJumpPopOperands { offset, value }) = extract_back_infallible(&mut cx);

        let jump = !value.is_truthy(&mut cx.scope);

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }
        Ok(None)
    }

    pub fn jmpfalsenp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpFalseNoPopOperands(ConditionalJumpNoPopOperands { offset, value }) = extract_back_infallible(&mut cx);

        let jump = !value.is_truthy(&mut cx.scope);

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmptruep(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpTruePopOperands(ConditionalJumpPopOperands { offset, value }) = extract_back_infallible(&mut cx);

        let jump = value.is_truthy(&mut cx.scope);

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmptruenp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpTrueNoPopOperands(ConditionalJumpNoPopOperands { offset, value }) = extract_back_infallible(&mut cx);

        let jump = value.is_truthy(&mut cx.scope);

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpnullishp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpNullishPopOperands(ConditionalJumpPopOperands { offset, value }) = extract_back_infallible(&mut cx);

        let jump = value.is_nullish();

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpnullishnp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpNullishNoPopOperands(ConditionalJumpNoPopOperands { offset, value }) = extract_back_infallible(&mut cx);

        let jump = value.is_nullish();

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpundefinedp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpUndefinedPopOperands(ConditionalJumpPopOperands { offset, value }) = extract_back_infallible(&mut cx);

        let jump = value.is_undefined();

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmpundefinednp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpUndefinedNoPopOperands(ConditionalJumpNoPopOperands { offset, value }) =
            extract_back_infallible(&mut cx);

        let jump = value.is_undefined();

        if jump {
            let ip = cx.frames.current_ip();
            cx.frames.set_ip(ip + offset);
        }

        Ok(None)
    }

    pub fn jmp(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let JmpOperands(offset) = extract_back_infallible(&mut cx);

        let ip = cx.frames.current_ip();
        cx.frames.set_ip(ip + offset);

        Ok(None)
    }

    pub fn storelocal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        fn binop(
            cx: &mut DispatchContext<'_>,
            id: BackLocalId,
            right: Value,
            op: impl FnOnce(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let left = cx.get_local(id);
            let res = op(left, right, &mut cx.scope)?;
            cx.set_local(id, res.into());
            cx.stack.push(res);
            Ok(())
        }

        fn prefix(
            cx: &mut DispatchContext<'_>,
            id: BackLocalId,
            op: impl FnOnce(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let left = cx.get_local(id);
            let left = Value::number(left.to_number(&mut cx.scope)?);
            let right = Value::number(1.0);
            let res = op(left, right, &mut cx.scope)?;
            cx.set_local(id, res.clone().into());
            cx.stack.push(res);
            Ok(())
        }

        fn postfix(
            cx: &mut DispatchContext<'_>,
            id: BackLocalId,
            op: impl FnOnce(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let left = cx.get_local(id);
            let left = Value::number(left.to_number(&mut cx.scope)?);
            let right = Value::number(1.0);
            let res = op(left, right, &mut cx.scope)?;
            cx.set_local(id, res.into());
            cx.stack.push(left);
            Ok(())
        }

        let StoreLocalOperands { local, kind } = extract_back_infallible(&mut cx);

        match kind {
            AssignKind::Assignment(value) => {
                cx.set_local(local, value);
                cx.push_stack(value);
            }
            AssignKind::AddAssignment(value) => binop(&mut cx, local, value, Value::add)?,
            AssignKind::SubAssignment(value) => binop(&mut cx, local, value, Value::sub)?,
            AssignKind::MulAssignment(value) => binop(&mut cx, local, value, Value::mul)?,
            AssignKind::DivAssignment(value) => binop(&mut cx, local, value, Value::div)?,
            AssignKind::RemAssignment(value) => binop(&mut cx, local, value, Value::rem)?,
            AssignKind::PowAssignment(value) => binop(&mut cx, local, value, Value::pow)?,
            AssignKind::ShlAssignment(value) => binop(&mut cx, local, value, Value::bitshl)?,
            AssignKind::ShrAssignment(value) => binop(&mut cx, local, value, Value::bitshr)?,
            AssignKind::UshrAssignment(value) => binop(&mut cx, local, value, Value::bitushr)?,
            AssignKind::BitAndAssignment(value) => binop(&mut cx, local, value, Value::bitand)?,
            AssignKind::BitOrAssignment(value) => binop(&mut cx, local, value, Value::bitor)?,
            AssignKind::BitXorAssignment(value) => binop(&mut cx, local, value, Value::bitxor)?,
            AssignKind::PrefixIncrement => prefix(&mut cx, local, Value::add)?,
            AssignKind::PrefixDecrement => prefix(&mut cx, local, Value::sub)?,
            AssignKind::PostfixIncrement => postfix(&mut cx, local, Value::add)?,
            AssignKind::PostfixDecrement => postfix(&mut cx, local, Value::sub)?,
        }

        Ok(None)
    }

    pub fn ldlocal(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let LdLocalOperands(id) = extract_back_infallible(&mut cx);
        let value = cx.get_local(id);

        cx.stack.push(value);
        Ok(None)
    }

    fn with_arraylit_elements(
        cx: &mut DispatchContext<'_>,
        len: usize,
        stack_values: usize,
        mut fun: impl FnMut(ArrayLiteralElement<DispatchContext<'_>>),
    ) -> Result<(), Unrooted> {
        let mut iter =
            ForwardSequence::<ArrayLiteralElement<DispatchContext<'_>>>::from_stack_count_len(cx, stack_values, len);
        while let Some(element) = iter.next_infallible(cx) {
            match element {
                ArrayLiteralElement::Single(value) => fun(ArrayLiteralElement::Single(value)),
                ArrayLiteralElement::Spread(source) => {
                    let len = source.length_of_array_like(&mut cx.scope)?;
                    for i in 0..len {
                        let value = source
                            .get_property(i.to_key(&mut cx.scope), &mut cx.scope)?
                            .root(&mut cx.scope);
                        fun(ArrayLiteralElement::Single(value));
                    }
                }
                ArrayLiteralElement::Hole(count) => fun(ArrayLiteralElement::Hole(count)),
            }
        }
        let truncate_to = cx.stack.len() - stack_values;
        cx.stack.truncate(truncate_to);

        debug_assert!(iter.next_infallible(cx).is_none());
        Ok(())
    }

    fn arraylit_holey(cx: &mut DispatchContext<'_>, len: usize, stack_values: usize) -> Result<Array, Unrooted> {
        let mut table = ArrayTable::new();
        with_arraylit_elements(cx, len, stack_values, |element| match element {
            ArrayLiteralElement::Single(value) => table.push(PropertyValue::static_default(value)),
            ArrayLiteralElement::Hole(hole) => table.resize(table.len() + hole),
            ArrayLiteralElement::Spread(..) => unreachable!(),
        })?;
        Ok(Array::from_table(&cx.scope, table))
    }

    fn arraylit_dense(cx: &mut DispatchContext<'_>, len: usize) -> Result<Array, Unrooted> {
        // Dense implies len == stack_values
        let mut new_elements = Vec::with_capacity(len);
        with_arraylit_elements(cx, len, len, |element| match element {
            ArrayLiteralElement::Single(value) => new_elements.push(PropertyValue::static_default(value)),
            ArrayLiteralElement::Spread(..) | ArrayLiteralElement::Hole(_) => unreachable!(),
        })?;
        Ok(Array::from_vec(new_elements, &cx.scope))
    }

    pub fn arraylit(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ArrayLiteralOperands { len, stack_values } = extract_back_infallible(&mut cx);

        // Split up into two functions as a non-holey array literal can be evaluated more efficiently
        let array = if len == stack_values {
            arraylit_dense(&mut cx, len.into())?
        } else {
            arraylit_holey(&mut cx, len.into(), stack_values.into())?
        };

        let handle = cx.scope.register(array);
        cx.stack.push(Value::object(handle));
        Ok(None)
    }

    pub fn objlit(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ObjectLiteralOperands { mut members } = extract_back_infallible(&mut cx);

        let obj = OrdObject::new(&cx.scope);
        while let Some(property) = members.next_infallible(&mut cx) {
            match property {
                ObjectProperty::StaticGetter { key, value } => {
                    let key = key.0.to_key(&mut cx.scope);
                    let ValueKind::Object(getter) = value.unpack() else {
                        unreachable!()
                    };

                    match obj.get_own_property_descriptor(key, &mut cx.scope)? {
                        Some(prop) => {
                            obj.set_property(key, prop.with_getter(getter), &mut cx.scope)?;
                        }
                        None => {
                            obj.set_property(key, PropertyValue::getter_default(getter), &mut cx.scope)?;
                        }
                    }
                }
                ObjectProperty::DynamicGetter { key, value } => {
                    let key = PropertyKey::from_value(&mut cx.scope, key)?;
                    let ValueKind::Object(getter) = value.unpack() else {
                        unreachable!()
                    };

                    match obj.get_own_property_descriptor(key, &mut cx.scope)? {
                        Some(prop) => {
                            obj.set_property(key, prop.with_getter(getter), &mut cx.scope)?;
                        }
                        None => {
                            obj.set_property(key, PropertyValue::getter_default(getter), &mut cx.scope)?;
                        }
                    }
                }
                ObjectProperty::StaticSetter { key, value } => {
                    let key = key.0.to_key(&mut cx.scope);
                    let ValueKind::Object(setter) = value.unpack() else {
                        unreachable!()
                    };

                    match obj.get_own_property_descriptor(key, &mut cx.scope)? {
                        Some(prop) => {
                            obj.set_property(key, prop.with_setter(setter), &mut cx.scope)?;
                        }
                        None => {
                            obj.set_property(key, PropertyValue::setter_default(setter), &mut cx.scope)?;
                        }
                    }
                }
                ObjectProperty::DynamicSetter { key, value } => {
                    let key = PropertyKey::from_value(&mut cx.scope, key)?;
                    let ValueKind::Object(setter) = value.unpack() else {
                        unreachable!()
                    };

                    match obj.get_own_property_descriptor(key, &mut cx.scope)? {
                        Some(prop) => {
                            obj.set_property(key, prop.with_setter(setter), &mut cx.scope)?;
                        }
                        None => {
                            obj.set_property(key, PropertyValue::setter_default(setter), &mut cx.scope)?;
                        }
                    }
                }
                ObjectProperty::Static { key, value } => {
                    let key = key.0.to_key(&mut cx.scope);
                    obj.set_property(key, PropertyValue::static_default(value), &mut cx.scope)?;
                }
                ObjectProperty::Dynamic { key, value } => {
                    let key = PropertyKey::from_value(&mut cx.scope, key)?;
                    obj.set_property(key, PropertyValue::static_default(value), &mut cx.scope)?;
                }
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

        members.commit(&mut cx);

        let handle = cx.scope.register(obj);
        cx.stack.push(handle.into());

        Ok(None)
    }

    pub fn assign_properties(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let AssignPropertiesOperands { target, mut members } = extract_back_infallible(&mut cx);

        while let Some(property) = members.next_infallible(&mut cx) {
            match property {
                ObjectProperty::Static { key, value } => {
                    let key = key.0.clone().to_key(&mut cx.scope);
                    target.set_property(key, PropertyValue::static_default(value), &mut cx.scope)?;
                }
                ObjectProperty::StaticGetter { key, value } => {
                    let key = key.0.clone().to_key(&mut cx.scope);
                    let ValueKind::Object(getter) = value.unpack() else {
                        unreachable!()
                    };

                    match target.get_own_property_descriptor(key, &mut cx.scope)? {
                        Some(prop) => {
                            target.set_property(key, prop.with_getter(getter), &mut cx.scope)?;
                        }
                        None => {
                            target.set_property(key, PropertyValue::getter_default(getter), &mut cx.scope)?;
                        }
                    }
                }
                ObjectProperty::DynamicGetter { key, value } => {
                    let key = PropertyKey::from_value(&mut cx.scope, key)?;
                    let ValueKind::Object(getter) = value.unpack() else {
                        unreachable!()
                    };

                    match target.get_own_property_descriptor(key, &mut cx.scope)? {
                        Some(prop) => {
                            target.set_property(key, prop.with_getter(getter), &mut cx.scope)?;
                        }
                        None => {
                            target.set_property(key, PropertyValue::getter_default(getter), &mut cx.scope)?;
                        }
                    }
                }
                ObjectProperty::StaticSetter { key, value } => {
                    let key = key.0.clone().to_key(&mut cx.scope);
                    let ValueKind::Object(setter) = value.unpack() else {
                        unreachable!()
                    };

                    match target.get_own_property_descriptor(key, &mut cx.scope)? {
                        Some(prop) => {
                            target.set_property(key, prop.with_setter(setter), &mut cx.scope)?;
                        }
                        None => {
                            target.set_property(key, PropertyValue::setter_default(setter), &mut cx.scope)?;
                        }
                    }
                }
                ObjectProperty::DynamicSetter { key, value } => {
                    let key = PropertyKey::from_value(&mut cx.scope, key)?;
                    let ValueKind::Object(setter) = value.unpack() else {
                        unreachable!()
                    };

                    match target.get_own_property_descriptor(key, &mut cx.scope)? {
                        Some(prop) => {
                            target.set_property(key, prop.with_setter(setter), &mut cx.scope)?;
                        }
                        None => {
                            target.set_property(key, PropertyValue::setter_default(setter), &mut cx.scope)?;
                        }
                    }
                }
                ObjectProperty::Dynamic { key, value } => {
                    let key = PropertyKey::from_value(&mut cx.scope, key)?;
                    target.set_property(key, PropertyValue::static_default(value), &mut cx.scope)?;
                }
                ObjectProperty::Spread(_) => unimplemented!("spread operator in AssignProperties"),
            }
        }
        members.commit(&mut cx);

        Ok(None)
    }

    pub fn staticpropertyaccess(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let StaticPropertyAccessOperands {
            target,
            ident: SymbolConstantWide(ident),
        } = extract_back_infallible(&mut cx);

        let value = target.get_property(ident.to_key(&mut cx.scope), &mut cx.scope)?;
        cx.push_stack(value);
        Ok(None)
    }

    /// Shared logic between static and dynamic property assignment
    fn property_assign(
        mut cx: DispatchContext<'_>,
        key: PropertyKey,
        target: Value,
        kind: AssignKind<DispatchContext<'_>>,
    ) -> Result<Option<HandleResult>, Unrooted> {
        fn binop(
            cx: &mut DispatchContext<'_>,
            target: Value,
            right: Value,
            key: PropertyKey,
            op: fn(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let prop = target.get_property(key, &mut cx.scope)?.root(&mut cx.scope);
            let res = op(prop, right, &mut cx.scope)?;
            target.set_property(key, PropertyValue::static_default(res.clone()), &mut cx.scope)?;
            cx.stack.push(res);
            Ok(())
        }

        fn postfix(
            cx: &mut DispatchContext<'_>,
            target: Value,
            key: PropertyKey,
            op: fn(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let prop = target.get_property(key, &mut cx.scope)?.root(&mut cx.scope);
            let prop = Value::number(prop.to_number(&mut cx.scope)?);
            let one = Value::number(1.0);
            let res = op(prop, one, &mut cx.scope)?;
            target.set_property(key, PropertyValue::static_default(res), &mut cx.scope)?;
            cx.stack.push(prop);
            Ok(())
        }

        fn prefix(
            cx: &mut DispatchContext<'_>,
            target: Value,
            key: PropertyKey,
            op: fn(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let prop = target.get_property(key, &mut cx.scope)?.root(&mut cx.scope);
            let prop = Value::number(prop.to_number(&mut cx.scope)?);
            let one = Value::number(1.0);
            let res = op(prop, one, &mut cx.scope)?;
            target.set_property(key, PropertyValue::static_default(res.clone()), &mut cx.scope)?;
            cx.stack.push(res);
            Ok(())
        }

        match kind {
            AssignKind::Assignment(value) => {
                let value = value.root(&mut cx.scope);
                target.set_property(key, PropertyValue::static_default(value), &mut cx.scope)?;
                cx.stack.push(value);
            }
            AssignKind::AddAssignment(value) => binop(&mut cx, target, value, key, Value::add)?,
            AssignKind::SubAssignment(value) => binop(&mut cx, target, value, key, Value::sub)?,
            AssignKind::MulAssignment(value) => binop(&mut cx, target, value, key, Value::mul)?,
            AssignKind::DivAssignment(value) => binop(&mut cx, target, value, key, Value::div)?,
            AssignKind::RemAssignment(value) => binop(&mut cx, target, value, key, Value::rem)?,
            AssignKind::PowAssignment(value) => binop(&mut cx, target, value, key, Value::pow)?,
            AssignKind::ShlAssignment(value) => binop(&mut cx, target, value, key, Value::bitshl)?,
            AssignKind::ShrAssignment(value) => binop(&mut cx, target, value, key, Value::bitshr)?,
            AssignKind::UshrAssignment(value) => binop(&mut cx, target, value, key, Value::bitushr)?,
            AssignKind::BitAndAssignment(value) => binop(&mut cx, target, value, key, Value::bitand)?,
            AssignKind::BitOrAssignment(value) => binop(&mut cx, target, value, key, Value::bitor)?,
            AssignKind::BitXorAssignment(value) => binop(&mut cx, target, value, key, Value::bitxor)?,
            AssignKind::PrefixIncrement => prefix(&mut cx, target, key, Value::add)?,
            AssignKind::PrefixDecrement => prefix(&mut cx, target, key, Value::sub)?,
            AssignKind::PostfixIncrement => postfix(&mut cx, target, key, Value::add)?,
            AssignKind::PostfixDecrement => postfix(&mut cx, target, key, Value::sub)?,
        };

        Ok(None)
    }

    pub fn staticpropertyassign(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let StaticPropertyAssignOperands {
            target,
            kind,
            key: SymbolConstantWide(key),
        } = extract_back_infallible(&mut cx);

        let key = key.to_key(&mut cx.scope);
        property_assign(cx, key, target, kind)
    }

    pub fn dynamicpropertyassign(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let DynamicPropertyAssignOperands { target, kind, key } = extract_back_infallible(&mut cx);

        let key = PropertyKey::from_value(&mut cx.scope, key)?;
        property_assign(cx, key, target, kind)
    }

    pub fn dynamicpropertyaccess(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let DynamicPropertyAccessOperands { key, target } = extract_back_infallible(&mut cx);

        let key = PropertyKey::from_value(&mut cx, key)?;

        let value = target.get_property(key, &mut cx.scope)?;
        cx.push_stack(value);
        Ok(None)
    }

    pub fn ldlocalext(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let LdLocalExtOperands(id) = extract_back_infallible(&mut cx);

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
        let StoreLocalExtOperands { local, kind } = extract_back_infallible(&mut cx);

        fn binop(
            cx: &mut DispatchContext<'_>,
            external: ExternalId,
            right: Value,
            op: fn(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Value> {
            let external = cx.scope.get_external(external);
            let left = external.inner(&cx.scope);
            let res = op(left, right, &mut cx.scope)?;
            assign_to_external(&mut cx.scope, external, res);
            cx.stack.push(res);
            Ok(())
        }

        fn prefix(
            cx: &mut DispatchContext<'_>,
            id: ExternalId,
            op: fn(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let external = cx.get_external(id);
            let left = external.inner(&cx.scope);
            let right = Value::number(1.0);
            let res = op(left, right, &mut cx.scope)?;
            assign_to_external(&mut cx.scope, external, res);
            cx.stack.push(res);
            Ok(())
        }

        fn postfix(
            cx: &mut DispatchContext<'_>,
            id: ExternalId,
            op: fn(Value, Value, &mut LocalScope<'_>) -> Result<Value, Value>,
        ) -> Result<(), Unrooted> {
            let external = cx.get_external(id);
            let left = external.inner(&cx.scope);
            let left = Value::number(left.to_number(&mut cx.scope)?);
            let right = Value::number(1.0);
            let res = op(left, right, &mut cx.scope)?;
            assign_to_external(&mut cx.scope, external, res);
            cx.stack.push(left);
            Ok(())
        }

        match kind {
            AssignKind::Assignment(value) => {
                let value = value.root(&mut cx.scope);
                let external = cx.scope.get_external(local);
                assign_to_external(&mut cx.scope, external, value);
                cx.stack.push(value);
            }
            AssignKind::AddAssignment(value) => binop(&mut cx, local, value, Value::add)?,
            AssignKind::SubAssignment(value) => binop(&mut cx, local, value, Value::sub)?,
            AssignKind::MulAssignment(value) => binop(&mut cx, local, value, Value::mul)?,
            AssignKind::DivAssignment(value) => binop(&mut cx, local, value, Value::div)?,
            AssignKind::RemAssignment(value) => binop(&mut cx, local, value, Value::rem)?,
            AssignKind::PowAssignment(value) => binop(&mut cx, local, value, Value::pow)?,
            AssignKind::ShlAssignment(value) => binop(&mut cx, local, value, Value::bitshl)?,
            AssignKind::ShrAssignment(value) => binop(&mut cx, local, value, Value::bitshr)?,
            AssignKind::UshrAssignment(value) => binop(&mut cx, local, value, Value::bitushr)?,
            AssignKind::BitAndAssignment(value) => binop(&mut cx, local, value, Value::bitand)?,
            AssignKind::BitOrAssignment(value) => binop(&mut cx, local, value, Value::bitor)?,
            AssignKind::BitXorAssignment(value) => binop(&mut cx, local, value, Value::bitxor)?,
            AssignKind::PrefixIncrement => prefix(&mut cx, local, Value::add)?,
            AssignKind::PrefixDecrement => prefix(&mut cx, local, Value::sub)?,
            AssignKind::PostfixIncrement => postfix(&mut cx, local, Value::add)?,
            AssignKind::PostfixDecrement => postfix(&mut cx, local, Value::sub)?,
        }

        Ok(None)
    }

    pub fn try_block(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let mut compute_dist_ip = || {
            let distance = extract_back_infallible::<_, OptionDiscriminatedByte<u16>>(&mut cx).0?;
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
        let ThrowOperands { value } = extract_back_infallible(&mut cx);
        Err(value)
    }

    pub fn type_of(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let TypeofOperands { value } = extract_back_infallible(&mut cx);
        let ty = value.type_of(&cx.scope).as_value();
        cx.stack.push(ty);
        Ok(None)
    }

    pub fn type_of_ident(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let TypeofIdentOperands(SymbolConstantWide(ident)) = extract_back_infallible(&mut cx);
        let prop = cx
            .global
            .get_property(ident.to_key(&mut cx.scope), &mut cx.scope)?
            .root(&mut cx.scope);

        let ty = prop.type_of(&cx.scope).as_value();
        cx.stack.push(ty);
        Ok(None)
    }

    pub fn yield_(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let YieldOperands { value } = extract_back_infallible(&mut cx);
        Ok(Some(HandleResult::Yield(value)))
    }

    pub fn await_(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let AwaitOperands { value } = extract_back_infallible(&mut cx);
        Ok(Some(HandleResult::Await(value)))
    }

    pub fn import_dyn(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ImportDynOperands { value } = extract_back_infallible(&mut cx);

        let _ret = match cx.params.dynamic_import_callback {
            Some(cb) => cb(&mut cx, value)?,
            None => throw!(cx, Error, "Dynamic imports are disabled for this context"),
        };

        // TODO: dynamic imports are currently statements, making them useless
        // TODO: make them an expression and push ret on stack

        Ok(None)
    }

    pub fn import_static(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ImportStaticOperands {
            kind,
            local,
            path: SymbolConstantWide(path),
        } = extract_back_infallible(&mut cx);

        let value = match cx.params.static_import_callback {
            Some(cb) => cb(&mut cx, kind, path.into())?,
            None => throw!(cx, Error, "Static imports are disabled for this context."),
        };

        cx.set_local(local, value);

        Ok(None)
    }

    pub fn export_default(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ExportDefaultOperands { value } = extract_back_infallible(&mut cx);

        match cx.frames.current_state_mut() {
            FrameState::Module(module) => {
                module.default = Some(value);
            }
            _ => throw!(cx, Error, "Export is only available at the top level in modules"),
        }

        Ok(None)
    }

    pub fn export_named(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ExportNamedOperands { mut members } = extract_back_infallible(&mut cx);

        while let Some(prop) = members.next_infallible(&mut cx) {
            let (ident, value) = match prop {
                ExportProperty::Local { local, export_name } => (export_name.into(), cx.get_local(local).into()),
                ExportProperty::Global { ident } => {
                    let key = ident.to_key(&mut cx.scope);
                    let value = cx.global.get_property(key, &mut cx.scope)?;
                    (ident.into(), value)
                }
            };

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
        let BindThisOperands { value } = extract_back_infallible(&mut cx);
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
        let CallSymbolIteratorOperands { value } = extract_back_infallible(&mut cx);
        let symbol_iterator = cx.statics.symbol_iterator;
        let iterable = value
            .get_property(symbol_iterator.to_key(&mut cx.scope), &mut cx.scope)?
            .root(&mut cx.scope);
        let iterator = iterable.apply(This::bound(value), CallArgs::empty(), &mut cx.scope)?;
        cx.push_stack(iterator);
        Ok(None)
    }

    pub fn call_for_in_iterator(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ForInIteratorOperands { value } = extract_back_infallible(&mut cx);

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
        let DeletePropertyDynamicOperands { key, target } = extract_back_infallible(&mut cx);
        let key = PropertyKey::from_value(&mut cx, key)?;
        let value = target.delete_property(key, &mut cx.scope)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value.root(&mut cx.scope).unpack(), ValueKind::Undefined(..));
        cx.stack.push(Value::boolean(did_delete));
        Ok(None)
    }

    pub fn delete_property_static(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let DeletePropertyStaticOperands { key, target } = extract_back_infallible(&mut cx);
        let value = target.delete_property(key.0.to_key(&mut cx.scope), &mut cx.scope)?;

        // TODO: not correct, as `undefined` might have been the actual value
        let did_delete = !matches!(value.root(&mut cx.scope).unpack(), ValueKind::Undefined(..));
        cx.stack.push(Value::boolean(did_delete));
        Ok(None)
    }

    pub fn objdestruct(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ObjectDestructuringOperands {
            target,
            mut members,
            rest_local_id: OptionNoneMax(rest_local_id),
        } = extract_back_infallible(&mut cx);

        let mut idents = Vec::<JsString>::new();

        // let mut iter = BackwardSequence::<(bool, NumberWConstant, IdentW)>::new_u16(&mut cx);
        while let Some(ObjectDestructuringMember {
            has_default,
            id: NumberConstantWide(id),
            key: SymbolConstantWide(key),
        }) = members.next_infallible(&mut cx)
        {
            if rest_local_id.is_some() {
                idents.push(key.into());
            }

            let mut prop = target
                .get_property(key.to_key(&mut cx.scope), &mut cx.scope)?
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

        if let Some(rest_id) = rest_local_id {
            let keys = target
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
                let key = key.to_key(&mut cx.scope);
                let value = target.get_property(key, &mut cx.scope)?.root(&mut cx.scope);
                rest.set_property(key, PropertyValue::static_default(value), &mut cx.scope)?;
            }

            cx.set_local(rest_id.into(), Value::object(rest).into());
        }

        Ok(None)
    }

    pub fn arraydestruct(mut cx: DispatchContext<'_>) -> Result<Option<HandleResult>, Unrooted> {
        let ArrayDestructuringOperands { array, members } = extract_back_infallible(&mut cx);

        let mut members = <_ as IteratorWith<&mut DispatchContext<'_>>>::enumerate(members);

        while let Some((i, OptionDiscriminatedByte(member))) = members.next_infallible(&mut cx) {
            if let Some(ArrayDestructuringMember {
                has_default,
                id: NumberConstantWide(id),
            }) = member
            {
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
        let IntrinsicOperands(op) = extract_back_infallible(&mut cx);

        #[inline(always)]
        fn binop_numbers_to_f64(left: Value, right: Value) -> (f64, f64) {
            match (left.unpack(), right.unpack()) {
                (ValueKind::Number(l), ValueKind::Number(r)) => (l.0, r.0),
                _ => unreachable!(),
            }
        }

        #[inline(always)]
        fn number_binop_number(left: Value, right: Value, scope: &mut LocalScope<'_>, op: fn(f64, f64) -> f64) {
            let (left, right) = binop_numbers_to_f64(left, right);
            scope.stack.push(Value::number(op(left, right)));
        }

        #[inline(always)]
        fn number_binop_bool(left: Value, right: Value, scope: &mut LocalScope<'_>, op: fn(f64, f64) -> bool) {
            let (left, right) = binop_numbers_to_f64(left, right);
            let res = op(left, right);
            scope.stack.push(Value::boolean(res));
        }

        #[inline(always)]
        fn number_binop_i32(left: Value, right: Value, scope: &mut LocalScope<'_>, op: fn(i32, i32) -> i32) {
            let (left, right) = binop_numbers_to_f64(left, right);
            let left = left as i64 as i32;
            let right = right as i64 as i32;
            let res = op(left, right);
            scope.stack.push(Value::number(res as f64));
        }

        #[inline(always)]
        fn number_binop_u32(left: Value, right: Value, scope: &mut LocalScope<'_>, op: fn(u32, u32) -> u32) {
            let (left, right) = binop_numbers_to_f64(left, right);
            let left = left as i64 as u32;
            let right = right as i64 as u32;
            let res = op(left, right);
            scope.stack.push(Value::number(res as f64));
        }

        #[inline(always)]
        fn prefix(local: BackLocalId, cx: &mut DispatchContext<'_>, op: fn(f64) -> f64) {
            let value = match cx.get_local(local).unpack() {
                ValueKind::Number(n) => n.0,
                _ => unreachable!(),
            };
            let res = Value::number(op(value));
            cx.set_local(local, res.into());
            cx.stack.push(res);
        }

        #[inline(always)]
        fn postfix(local: BackLocalId, cx: &mut DispatchContext<'_>, op: fn(f64) -> f64) {
            let value = match cx.get_local(local).unpack() {
                ValueKind::Number(n) => n.0,
                _ => unreachable!(),
            };
            let res = Value::number(op(value));
            cx.set_local(local, res.into());
            cx.stack.push(Value::number(value));
        }

        #[inline(always)]
        fn number_f64_binop_bool(left: Value, right: f64, scope: &mut LocalScope<'_>, op: fn(f64, f64) -> bool) {
            let left = match left.unpack() {
                ValueKind::Number(n) => n.0,
                _ => unreachable!(),
            };
            let res = op(left, right);
            scope.stack.push(Value::boolean(res));
        }

        fn fn_call_impl(
            IntrinsicCallOperands { argc }: IntrinsicCallOperands,
            func: ObjectId,
            global_key: Symbol,
            object_key: Symbol,
            cx: &mut DispatchContext<'_>,
        ) -> Result<(), Unrooted> {
            let args = cx.drain_stack_rooted(argc.into()).collect::<CallArgs>();

            if likely(cx.builtins_purity()) {
                // Fast path: call builtin directly
                let result = func.apply(This::default(), args, &mut cx.scope)?;
                cx.push_stack(result);
            } else {
                // Builtins impure, fallback to slow dynamic property lookup
                for arg in &args {
                    cx.scope.add(arg.clone());
                }

                let k = cx
                    .global
                    .clone()
                    .get_property(global_key.to_key(&mut cx.scope), &mut cx.scope)?
                    .root(&mut cx.scope);
                let fun = k
                    .get_property(object_key.to_key(&mut cx.scope), &mut cx.scope)?
                    .root(&mut cx.scope);
                let result = fun.apply(This::default(), args, &mut cx.scope)?;
                cx.push_stack(result);
            }

            Ok(())
        }
        macro_rules! fn_call {
            ($call:expr, $func:ident, $global_key:ident.$object_key:ident) => {
                fn_call_impl(
                    $call,
                    cx.statics.$func,
                    sym::$global_key,
                    sym::$object_key,
                    &mut cx,
                )?
            };
        }

        match op {
            IntrinsicKind::AddNumLR(left, right) => number_binop_number(left, right, &mut cx.scope, f64::add),
            IntrinsicKind::SubNumLR(left, right) => number_binop_number(left, right, &mut cx.scope, f64::sub),
            IntrinsicKind::MulNumLR(left, right) => number_binop_number(left, right, &mut cx.scope, f64::mul),
            IntrinsicKind::DivNumLR(left, right) => number_binop_number(left, right, &mut cx.scope, f64::div),
            IntrinsicKind::RemNumLR(left, right) => number_binop_number(left, right, &mut cx.scope, f64::rem),
            IntrinsicKind::PowNumLR(left, right) => number_binop_number(left, right, &mut cx.scope, f64::powf),
            IntrinsicKind::GtNumLR(left, right) => number_binop_bool(left, right, &mut cx.scope, |l, r| l > r),
            IntrinsicKind::GeNumLR(left, right) => number_binop_bool(left, right, &mut cx.scope, |l, r| l >= r),
            IntrinsicKind::LtNumLR(left, right) => number_binop_bool(left, right, &mut cx.scope, |l, r| l < r),
            IntrinsicKind::LeNumLR(left, right) => number_binop_bool(left, right, &mut cx.scope, |l, r| l <= r),
            IntrinsicKind::EqNumLR(left, right) => number_binop_bool(left, right, &mut cx.scope, |l, r| l == r),
            IntrinsicKind::NeNumLR(left, right) => number_binop_bool(left, right, &mut cx.scope, |l, r| l != r),
            IntrinsicKind::BitOrNumLR(left, right) => number_binop_i32(left, right, &mut cx.scope, |l, r| l | r),
            IntrinsicKind::BitXorNumLR(left, right) => number_binop_i32(left, right, &mut cx.scope, |l, r| l ^ r),
            IntrinsicKind::BitAndNumLR(left, right) => number_binop_i32(left, right, &mut cx.scope, |l, r| l & r),
            IntrinsicKind::BitShlNumLR(left, right) => number_binop_i32(left, right, &mut cx.scope, |l, r| l << r),
            IntrinsicKind::BitShrNumLR(left, right) => number_binop_i32(left, right, &mut cx.scope, |l, r| l >> r),
            IntrinsicKind::BitUshrNumLR(left, right) => number_binop_u32(left, right, &mut cx.scope, |l, r| l >> r),
            IntrinsicKind::PostfixIncLocalNum(local) => postfix(local, &mut cx, |v| v + 1.0),
            IntrinsicKind::PostfixDecLocalNum(local) => postfix(local, &mut cx, |v| v - 1.0),
            IntrinsicKind::PrefixIncLocalNum(local) => prefix(local, &mut cx, |v| v + 1.0),
            IntrinsicKind::PrefixDecLocalNum(local) => prefix(local, &mut cx, |v| v - 1.0),
            IntrinsicKind::GtNumLConstR(left, NumberInline8(right))
            | IntrinsicKind::GtNumLConstR32(left, NumberInline32(right)) => {
                number_f64_binop_bool(left, right, &mut cx.scope, |l, r| l > r)
            }
            IntrinsicKind::GeNumLConstR(left, NumberInline8(right))
            | IntrinsicKind::GeNumLConstR32(left, NumberInline32(right)) => {
                number_f64_binop_bool(left, right, &mut cx.scope, |l, r| l >= r)
            }
            IntrinsicKind::LtNumLConstR(left, NumberInline8(right))
            | IntrinsicKind::LtNumLConstR32(left, NumberInline32(right)) => {
                number_f64_binop_bool(left, right, &mut cx.scope, |l, r| l < r)
            }
            IntrinsicKind::LeNumLConstR(left, NumberInline8(right))
            | IntrinsicKind::LeNumLConstR32(left, NumberInline32(right)) => {
                number_f64_binop_bool(left, right, &mut cx.scope, |l, r| l <= r)
            }
            IntrinsicKind::Exp(call) => fn_call!(call, math_exp, Math.exp),
            IntrinsicKind::Log2(call) => fn_call!(call, math_log2, Math.log2),
            IntrinsicKind::Expm1(call) => fn_call!(call, math_expm1, Math.expm1),
            IntrinsicKind::Cbrt(call) => fn_call!(call, math_cbrt, Math.cbrt),
            IntrinsicKind::Clz32(call) => fn_call!(call, math_clz32, Math.clz32),
            IntrinsicKind::Atanh(call) => fn_call!(call, math_atanh, Math.atanh),
            IntrinsicKind::Atan2(call) => fn_call!(call, math_atan2, Math.atan2),
            IntrinsicKind::Round(call) => fn_call!(call, math_round, Math.round),
            IntrinsicKind::Acosh(call) => fn_call!(call, math_acosh, Math.acosh),
            IntrinsicKind::Abs(call) => fn_call!(call, math_abs, Math.abs),
            IntrinsicKind::Sinh(call) => fn_call!(call, math_sinh, Math.sinh),
            IntrinsicKind::Sin(call) => fn_call!(call, math_sin, Math.sin),
            IntrinsicKind::Ceil(call) => fn_call!(call, math_ceil, Math.ceil),
            IntrinsicKind::Tan(call) => fn_call!(call, math_tan, Math.tan),
            IntrinsicKind::Trunc(call) => fn_call!(call, math_trunc, Math.trunc),
            IntrinsicKind::Asinh(call) => fn_call!(call, math_asinh, Math.asinh),
            IntrinsicKind::Log10(call) => fn_call!(call, math_log10, Math.log10),
            IntrinsicKind::Asin(call) => fn_call!(call, math_asin, Math.asin),
            IntrinsicKind::Random(call) => fn_call!(call, math_random, Math.random),
            IntrinsicKind::Log1p(call) => fn_call!(call, math_log1p, Math.log1p),
            IntrinsicKind::Sqrt(call) => fn_call!(call, math_sqrt, Math.sqrt),
            IntrinsicKind::Atan(call) => fn_call!(call, math_atan, Math.atan),
            IntrinsicKind::Cos(call) => fn_call!(call, math_cos, Math.cos),
            IntrinsicKind::Tanh(call) => fn_call!(call, math_tanh, Math.tanh),
            IntrinsicKind::Log(call) => fn_call!(call, math_log, Math.log),
            IntrinsicKind::Floor(call) => fn_call!(call, math_floor, Math.floor),
            IntrinsicKind::Cosh(call) => fn_call!(call, math_cosh, Math.cosh),
            IntrinsicKind::Acos(call) => fn_call!(call, math_acos, Math.acos),
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
        Instruction::String => handlers::string_constant(cx),
        Instruction::Boolean => handlers::boolean_constant(cx),
        Instruction::Number => handlers::number_constant(cx),
        Instruction::Regex => handlers::regex_constant(cx),
        Instruction::Null => handlers::null_constant(cx),
        Instruction::Undefined => handlers::undefined_constant(cx),
        Instruction::Function => handlers::function_constant(cx),
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
        Instruction::BindThis => handlers::bindthis(cx),
        Instruction::This => handlers::this(cx),
        Instruction::StaticPropAssign => handlers::staticpropertyassign(cx),
        Instruction::DynamicPropAssign => handlers::dynamicpropertyassign(cx),
        Instruction::LdLocalExt => handlers::ldlocalext(cx),
        Instruction::StoreLocalExt => handlers::storelocalext(cx),
        Instruction::StrictEq => handlers::strict_eq(cx),
        Instruction::StrictNe => handlers::strict_ne(cx),
        Instruction::Try => handlers::try_block(cx),
        Instruction::PopTry => handlers::pop_try(cx),
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
        Instruction::NewTarget => handlers::new_target(cx),
        Instruction::Nop => Ok(None),
    }
}
