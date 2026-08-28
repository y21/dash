use std::convert::Infallible;

use dash_regex::Regex;

use crate::compiler::constant::{BooleanConstant, FunctionConstant, NumberConstant, RegexConstant, SymbolConstant};
use crate::compiler::external::ExternalId;
use crate::compiler::extract::{
    BackwardSequence, ExtractBack, ExtractFront, ExtractSource, ForwardSequence, extract_back_infallible,
    extract_front_infallible,
};
use crate::compiler::instruction::IntrinsicOperation;
use crate::compiler::scope::BackLocalId;
use crate::compiler::{ExportPropertyKind, FunctionCallKind, ObjectMemberKind, StaticImportKind};
use crate::interner::Symbol;

// HELPERS
impl<S: ExtractSource> ExtractBack<S> for u16 {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(cx.fetch_u16())
    }
}

impl<S: ExtractSource> ExtractBack<S> for i16 {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(cx.fetch_u16().cast_signed())
    }
}

impl<S: ExtractSource> ExtractBack<S> for u8 {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(cx.fetch_u8())
    }
}

impl<S: ExtractSource> ExtractBack<S> for bool {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(cx.fetch_u8() == 1)
    }
}

pub enum AssignKind<S: ExtractSource> {
    Assignment(S::Unrooted),
    AddAssignment(S::Value),
    SubAssignment(S::Value),
    MulAssignment(S::Value),
    DivAssignment(S::Value),
    RemAssignment(S::Value),
    PowAssignment(S::Value),
    ShlAssignment(S::Value),
    ShrAssignment(S::Value),
    UshrAssignment(S::Value),
    BitAndAssignment(S::Value),
    BitOrAssignment(S::Value),
    BitXorAssignment(S::Value),
    PrefixIncrement,
    PostfixIncrement,
    PrefixDecrement,
    PostfixDecrement,
}

impl<S: ExtractSource> ExtractBack<S> for AssignKind<S> {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let kind = super::instruction::AssignKind::from_repr(cx.fetch_u8()).unwrap();

        match kind {
            super::instruction::AssignKind::Assignment => Ok(Self::Assignment(cx.pop_stack())),
            super::instruction::AssignKind::AddAssignment => Ok(Self::AddAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::SubAssignment => Ok(Self::SubAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::MulAssignment => Ok(Self::MulAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::DivAssignment => Ok(Self::DivAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::RemAssignment => Ok(Self::RemAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::PowAssignment => Ok(Self::PowAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::ShlAssignment => Ok(Self::ShlAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::ShrAssignment => Ok(Self::ShrAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::UshrAssignment => Ok(Self::UshrAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::BitAndAssignment => Ok(Self::BitAndAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::BitOrAssignment => Ok(Self::BitOrAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::BitXorAssignment => Ok(Self::BitXorAssignment(cx.pop_stack_rooted())),
            super::instruction::AssignKind::PrefixIncrement => Ok(Self::PrefixIncrement),
            super::instruction::AssignKind::PostfixIncrement => Ok(Self::PostfixIncrement),
            super::instruction::AssignKind::PrefixDecrement => Ok(Self::PrefixDecrement),
            super::instruction::AssignKind::PostfixDecrement => Ok(Self::PostfixDecrement),
        }
    }
}

impl<S: ExtractSource> ExtractBack<S> for FunctionCallKind {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(FunctionCallKind::from_repr(cx.fetch_u8()).unwrap())
    }
}

impl<S: ExtractSource> ExtractBack<S> for BackLocalId {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(BackLocalId(cx.fetch_u16()))
    }
}

/// Wrapper type for `Option<T>` that treats a max int value as `None`
pub struct OptionNoneMax<T>(pub Option<T>);

impl<S: ExtractSource> ExtractBack<S> for OptionNoneMax<BackLocalId> {
    type Exception = Infallible;

    fn extract_back(source: &mut S) -> Result<Self, Self::Exception> {
        let value = source.fetch_u16();
        if value == u16::MAX {
            Ok(Self(None))
        } else {
            Ok(Self(Some(BackLocalId(value))))
        }
    }
}

impl<S: ExtractSource> ExtractBack<S> for ExternalId {
    type Exception = Infallible;

    fn extract_back(source: &mut S) -> Result<Self, Self::Exception> {
        Ok(ExternalId(source.fetch_u16()))
    }
}

/// Wrapper type for `Option<T>` that encodes a discriminant byte (0=None, 1=Some), then the data
pub struct OptionDiscriminatedByte<T>(pub Option<T>);

impl<S: ExtractSource, T: ExtractBack<S>> ExtractBack<S> for OptionDiscriminatedByte<T> {
    type Exception = T::Exception;

    fn extract_back(source: &mut S) -> Result<Self, Self::Exception> {
        let discriminant = source.fetch_u8();
        match discriminant {
            0 => Ok(Self(None)),
            1 => T::extract_back(source).map(|v| Self(Some(v))),
            _ => panic!("Invalid discriminant for OptionDiscriminatedByte"),
        }
    }
}

pub struct TryCatchDepth(pub u16);

impl<S: ExtractSource> ExtractBack<S> for TryCatchDepth {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let depth = cx.fetch_u16();
        Ok(Self(depth))
    }
}

pub struct SymbolConstantWide(pub Symbol);

impl<S: ExtractSource> ExtractBack<S> for SymbolConstantWide {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let id = cx.fetch_u16();
        Ok(Self(cx.constants().symbols[SymbolConstant(id)].into()))
    }
}

pub struct BooleanConstantWide(pub bool);

impl<S: ExtractSource> ExtractBack<S> for BooleanConstantWide {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let id = cx.fetch_u16();
        Ok(Self(cx.constants().booleans[BooleanConstant(id)]))
    }
}

pub struct NumberConstantWide(pub f64);

impl<S: ExtractSource> ExtractBack<S> for NumberConstantWide {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let id = cx.fetch_u16();
        Ok(Self(cx.constants().numbers[NumberConstant(id)]))
    }
}

/// 8-bit f64 value inlined into the bytecode
pub struct NumberInline8(pub f64);

impl<S: ExtractSource> ExtractBack<S> for NumberInline8 {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(Self(cx.fetch_u8() as f64))
    }
}

/// 32-bit f64 value inlined into the bytecode
pub struct NumberInline32(pub f64);

impl<S: ExtractSource> ExtractBack<S> for NumberInline32 {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(Self(cx.fetch_u32() as f64))
    }
}

pub struct BinaryOperator<S: ExtractSource> {
    pub left: S::Value,
    pub right: S::Value,
}

impl<S: ExtractSource> ExtractBack<S> for BinaryOperator<S> {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let right = cx.pop_stack_rooted();
        let left = cx.pop_stack_rooted();
        Ok(Self { left, right })
    }
}

pub struct ConditionalJumpNoPopOperands<S: ExtractSource> {
    pub offset: i16,
    pub value: S::Unrooted,
}

impl<S: ExtractSource> ExtractBack<S> for ConditionalJumpNoPopOperands<S> {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let offset = cx.fetch_u16().cast_signed();
        let value = cx.peek_stack();
        Ok(Self { offset, value })
    }
}

impl<S: ExtractSource> ExtractBack<S> for ObjectMemberKind {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(ObjectMemberKind::from_repr(cx.fetch_u8()).unwrap())
    }
}

impl<S: ExtractSource> ExtractBack<S> for StaticImportKind {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(StaticImportKind::from_repr(cx.fetch_u8()).unwrap())
    }
}

impl<S: ExtractSource> ExtractBack<S> for ExportPropertyKind {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(Self::from_repr(cx.fetch_u8()).unwrap())
    }
}

// OPERANDS
macro_rules! define_operand_struct {
    (
        type Exception = $ety:ty;
        struct $name:ident ( $( $vis:vis $fty:ty ),* );
    ) => {
        pub struct $name ($( $vis $fty ),*);
        impl<S: ExtractSource> ExtractBack<S> for $name {
            type Exception = $ety;

            fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
                Ok(Self($( <$fty as ExtractBack<S>>::extract_back(cx)? ),*))
            }
        }
    };
    (
        type Exception = $ety:ty;
        struct $name:ident { $( $vis:vis $fname:ident : $fty:ty ),* }
    ) => {
        pub struct $name { $( $vis $fname: $fty ),* }
        impl<S: ExtractSource> ExtractBack<S> for $name {
            type Exception = $ety;

            fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
                Ok(Self { $( $fname: <$fty as ExtractBack<S>>::extract_back(cx)? ),* })
            }
        }
    };
}

macro_rules! define_operand_struct_with_source {
    (
        type Exception = $ety:ty;
        struct $name:ident <$($gen_name:ident : $gen_bound:path),*> ( $( $vis:vis $fty:ty ),* );
    ) => {
        pub struct $name<S: ExtractSource> ($( $vis $fty ),*);
        impl<S: ExtractSource> ExtractBack<S> for $name<S> {
            type Exception = $ety;

            fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
                Ok(Self($( <$fty as ExtractBack<S>>::extract_back(cx)? ),*))
            }
        }
    };
    (
        type Exception = $ety:ty;
        struct $name:ident <$($gen_name:ident : $gen_bound:path),*> { $( $vis:vis $fname:ident : $fty:ty ),* }
    ) => {
        pub struct $name<S: ExtractSource> { $( $vis $fname: $fty ),* }
        impl<S: ExtractSource> ExtractBack<S> for $name<S> {
            type Exception = $ety;

            fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
                Ok(Self { $( $fname: <$fty as ExtractBack<S>>::extract_back(cx)? ),* })
            }
        }
    };
}

define_operand_struct! {
    type Exception = Infallible;
    struct StringConstantOperands(pub SymbolConstantWide);
}

define_operand_struct! {
    type Exception = Infallible;
    struct BooleanConstantOperands(pub BooleanConstantWide);
}

define_operand_struct! {
    type Exception = Infallible;
    struct NumberConstantOperands(pub NumberConstantWide);
}

pub struct RegexConstantOperands(pub RegexConstant);
impl<S: ExtractSource> ExtractBack<S> for RegexConstantOperands {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let id = cx.fetch_u16();
        Ok(Self(RegexConstant(id)))
    }
}

impl RegexConstantOperands {
    pub fn regex<'a>(&self, cx: &'a mut impl ExtractSource) -> (&'a Regex, Symbol) {
        let (ref regex, source) = cx.constants().regexes[self.0];
        (regex, source)
    }
}

pub struct FunctionConstantOperands(pub FunctionConstant);

impl<S: ExtractSource> ExtractBack<S> for FunctionConstantOperands {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let id = cx.fetch_u16();
        Ok(Self(FunctionConstant(id)))
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct AddOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct SubOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct MulOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct DivOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct RemOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct PowOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct BitorOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct BitxorOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct BitandOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct BitshlOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct BitshrOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct BitushrOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;

    struct BitnotOperands<S: ExtractSource>(pub S::Value);
}

define_operand_struct_with_source! {
    type Exception = Infallible;

    struct ObjectInOperands<S: ExtractSource> {
        pub target: S::Value,
        pub key: S::Value
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;

    struct InstanceofOperands<S: ExtractSource> {
        pub constructor: S::Value,
        pub value: S::Value
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct LtOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct LeOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct GtOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct GeOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct EqOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct NeOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct StrictEqOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct StrictNeOperands<S: ExtractSource>(pub BinaryOperator<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct NegOperands<S: ExtractSource>(pub S::Value);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct PosOperands<S: ExtractSource>(pub S::Value);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct NotOperands<S: ExtractSource>(pub S::Value);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct PopOperands<S: ExtractSource>(pub S::Unrooted);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct DelayedRetOperands<S: ExtractSource>(pub S::Unrooted);
}

define_operand_struct! {
    type Exception = Infallible;
    struct FinallyEndOperands(pub TryCatchDepth);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct RetOperands<S: ExtractSource> {
        pub tc_depth: TryCatchDepth,
        pub value: S::Value
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct LdGlobalOperands(pub SymbolConstantWide);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct StoreGlobalOperands<S: ExtractSource> {
        pub name: SymbolConstantWide,
        pub kind: AssignKind<S>
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct CallOperands {
        pub argc: u8,
        pub has_this: bool,
        pub function_call_kind: FunctionCallKind
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct ConditionalJumpPopOperands<S: ExtractSource> {
        pub offset: i16,
        pub value: S::Unrooted
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct JmpFalsePopOperands<S: ExtractSource>(pub ConditionalJumpPopOperands<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct JmpFalseNoPopOperands<S: ExtractSource>(pub ConditionalJumpNoPopOperands<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct JmpTruePopOperands<S: ExtractSource>(pub ConditionalJumpPopOperands<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct JmpTrueNoPopOperands<S: ExtractSource>(pub ConditionalJumpNoPopOperands<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct JmpNullishPopOperands<S: ExtractSource>(pub ConditionalJumpPopOperands<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct JmpNullishNoPopOperands<S: ExtractSource>(pub ConditionalJumpNoPopOperands<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct JmpUndefinedPopOperands<S: ExtractSource>(pub ConditionalJumpPopOperands<S>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct JmpUndefinedNoPopOperands<S: ExtractSource>(pub ConditionalJumpNoPopOperands<S>);
}

define_operand_struct! {
    type Exception = Infallible;
    struct JmpOperands(pub i16);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct StoreLocalOperands<S: ExtractSource> {
        pub local: BackLocalId,
        pub kind: AssignKind<S>
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct LdLocalOperands(pub BackLocalId);
}

define_operand_struct! {
    type Exception = Infallible;
    struct ArrayLiteralOperands {
        // TODO: store a forwardsequence, similar to object literals!
        pub len: u16,
        pub stack_values: u16
    }
}

pub enum ObjectProperty<S: ExtractSource> {
    StaticGetter { key: SymbolConstantWide, value: S::Value },
    DynamicGetter { key: S::Value, value: S::Value },
    StaticSetter { key: SymbolConstantWide, value: S::Value },
    DynamicSetter { key: S::Value, value: S::Value },
    Static { key: SymbolConstantWide, value: S::Value },
    Dynamic { key: S::Value, value: S::Value },
    Spread(S::Value),
}

impl<S: ExtractSource> ExtractFront<S> for ObjectProperty<S> {
    type Exception = Infallible;

    fn extract_front<U>(source: &mut S, seq: &mut ForwardSequence<U>) -> Result<Self, Self::Exception> {
        Ok(match extract_back_infallible(source) {
            ObjectMemberKind::Getter => {
                let key = extract_back_infallible(source);
                let value = extract_front_infallible(source, seq);
                Self::StaticGetter { key, value }
            }
            ObjectMemberKind::DynamicGetter => {
                let key = extract_front_infallible(source, seq);
                let value = extract_front_infallible(source, seq);
                Self::DynamicGetter { key, value }
            }
            ObjectMemberKind::Setter => {
                let key = extract_back_infallible(source);
                let value = extract_front_infallible(source, seq);
                Self::StaticSetter { key, value }
            }
            ObjectMemberKind::DynamicSetter => {
                let key = extract_front_infallible(source, seq);
                let value = extract_front_infallible(source, seq);
                Self::DynamicSetter { key, value }
            }
            ObjectMemberKind::Static => {
                let key = extract_back_infallible(source);
                let value = extract_front_infallible(source, seq);
                Self::Static { key, value }
            }
            ObjectMemberKind::Dynamic => {
                let key = extract_front_infallible(source, seq);
                let value = extract_front_infallible(source, seq);
                Self::Dynamic { key, value }
            }
            ObjectMemberKind::Spread => Self::Spread(extract_front_infallible(source, seq)),
        })
    }
}

pub struct ObjectLiteralOperands<S: ExtractSource> {
    pub members: ForwardSequence<ObjectProperty<S>>,
}

impl<S: ExtractSource> ExtractBack<S> for ObjectLiteralOperands<S> {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let len = cx.fetch_u16();
        let stack_count = cx.fetch_u16();
        Ok(Self {
            members: ForwardSequence::from_stack_count_len(cx, stack_count.into(), len.into()),
        })
    }
}

pub struct AssignPropertiesOperands<S: ExtractSource> {
    pub members: ForwardSequence<ObjectProperty<S>>,
    pub target: S::Value,
}

impl<S: ExtractSource> ExtractBack<S> for AssignPropertiesOperands<S> {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let len = cx.fetch_u16();
        let stack_count = cx.fetch_u16();
        let target = cx.pop_stack_rooted();
        Ok(Self {
            members: ForwardSequence::from_stack_count_len(cx, stack_count.into(), len.into()),
            target,
        })
    }
}

pub struct StaticPropertyAccessOperands<S: ExtractSource> {
    pub ident: SymbolConstantWide,
    pub target: S::Value,
}

impl<S: ExtractSource> ExtractBack<S> for StaticPropertyAccessOperands<S> {
    type Exception = Infallible;

    fn extract_back(source: &mut S) -> Result<Self, Self::Exception> {
        let ident = extract_back_infallible(source);
        let preserve_this: bool = extract_back_infallible(source);
        let target = if preserve_this {
            source.peek_stack_rooted()
        } else {
            source.pop_stack_rooted()
        };
        Ok(Self { ident, target })
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct StaticPropertyAssignOperands<S: ExtractSource> {
        pub kind: AssignKind<S>,
        pub target: S::Value,
        pub key: SymbolConstantWide
    }
}

pub struct DynamicPropertyAccessOperands<S: ExtractSource> {
    pub key: S::Value,
    pub target: S::Value,
}

impl<S: ExtractSource> ExtractBack<S> for DynamicPropertyAccessOperands<S> {
    type Exception = Infallible;

    fn extract_back(source: &mut S) -> Result<Self, Self::Exception> {
        let key = source.pop_stack_rooted();
        let preserve_this: bool = extract_back_infallible(source);
        let target = if preserve_this {
            source.peek_stack_rooted()
        } else {
            source.pop_stack_rooted()
        };
        Ok(Self { key, target })
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct DynamicPropertyAssignOperands<S: ExtractSource> {
        pub kind: AssignKind<S>,
        pub key: S::Value,
        pub target: S::Value
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct LdLocalExtOperands(pub ExternalId);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct StoreLocalExtOperands<S: ExtractSource> {
        pub local: ExternalId,
        pub kind: AssignKind<S>
    }
}

// TODO: try block

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct ThrowOperands<S: ExtractSource> {
        pub value: S::Unrooted
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct TypeofOperands<S: ExtractSource> {
        pub value: S::Value
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct TypeofIdentOperands(pub SymbolConstantWide);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct YieldOperands<S: ExtractSource> {
        pub value: S::Unrooted
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct AwaitOperands<S: ExtractSource> {
        pub value: S::Unrooted
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct ImportDynOperands<S: ExtractSource> {
        pub value: S::Value
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct ImportStaticOperands {
        pub kind: StaticImportKind,
        pub local: BackLocalId,
        pub path: SymbolConstantWide
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct ExportDefaultOperands<S: ExtractSource> {
        pub value: S::Unrooted
    }
}

pub enum ExportProperty {
    Local { local: BackLocalId, export_name: Symbol },
    Global { ident: Symbol },
}
impl<S: ExtractSource> ExtractBack<S> for ExportProperty {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        Ok(match extract_back_infallible(cx) {
            ExportPropertyKind::Local => {
                let local: BackLocalId = extract_back_infallible(cx);
                let ident: SymbolConstantWide = extract_back_infallible(cx);
                Self::Local {
                    local,
                    export_name: ident.0,
                }
            }
            ExportPropertyKind::Global => Self::Global {
                ident: extract_back_infallible::<_, SymbolConstantWide>(cx).0,
            },
        })
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct ExportNamedOperands(pub BackwardSequence<ExportProperty>);
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct BindThisOperands<S: ExtractSource> {
        pub value: S::Value
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct CallSymbolIteratorOperands<S: ExtractSource> {
        pub value: S::Value
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct ForInIteratorOperands<S: ExtractSource> {
        pub value: S::Value
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct DeletePropertyDynamicOperands<S: ExtractSource> {
        pub target: S::Value,
        pub key: S::Value
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct DeletePropertyStaticOperands<S: ExtractSource> {
        pub target: S::Value,
        pub key: SymbolConstantWide
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct ObjectDestructuringMember {
        pub has_default: bool,
        pub id: NumberConstantWide, // TODO: just encode the id directly?
        pub key: SymbolConstantWide
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct ObjectDestructuringOperands<S: ExtractSource> {
        pub rest_local_id: OptionNoneMax<BackLocalId>,
        pub target: S::Value,
        pub members: BackwardSequence<ObjectDestructuringMember>
    }
}

define_operand_struct! {
    type Exception = Infallible;
    struct ArrayDestructuringMember {
        pub has_default: bool,
        pub id: NumberConstantWide // TODO: just encode the id directly?
    }
}

define_operand_struct_with_source! {
    type Exception = Infallible;
    struct ArrayDestructuringOperands<S: ExtractSource> {
        pub array: S::Value,
        pub members: BackwardSequence<OptionDiscriminatedByte<ArrayDestructuringMember>>
    }
}

pub struct IntrinsicCallOperands {
    pub argc: u8,
}

pub enum IntrinsicKind<S: ExtractSource> {
    // TODO: these could all just be unrooted since we can use try_prim
    AddNumLR(S::Value, S::Value),
    SubNumLR(S::Value, S::Value),
    MulNumLR(S::Value, S::Value),
    DivNumLR(S::Value, S::Value),
    RemNumLR(S::Value, S::Value),
    PowNumLR(S::Value, S::Value),
    GtNumLR(S::Value, S::Value),
    GeNumLR(S::Value, S::Value),
    LtNumLR(S::Value, S::Value),
    LeNumLR(S::Value, S::Value),
    EqNumLR(S::Value, S::Value),
    NeNumLR(S::Value, S::Value),
    BitOrNumLR(S::Value, S::Value),
    BitXorNumLR(S::Value, S::Value),
    BitAndNumLR(S::Value, S::Value),
    BitShlNumLR(S::Value, S::Value),
    BitShrNumLR(S::Value, S::Value),
    BitUshrNumLR(S::Value, S::Value),
    PostfixIncLocalNum(S::Value),
    PostfixDecLocalNum(S::Value),
    PrefixIncLocalNum(S::Value),
    PrefixDecLocalNum(S::Value),
    GtNumLConstR(S::Value, NumberInline8),
    GeNumLConstR(S::Value, NumberInline8),
    LtNumLConstR(S::Value, NumberInline8),
    LeNumLConstR(S::Value, NumberInline8),
    GtNumLConstR32(S::Value, NumberInline32),
    GeNumLConstR32(S::Value, NumberInline32),
    LtNumLConstR32(S::Value, NumberInline32),
    LeNumLConstR32(S::Value, NumberInline32),
    Exp(IntrinsicCallOperands),
    Log2(IntrinsicCallOperands),
    Expm1(IntrinsicCallOperands),
    Cbrt(IntrinsicCallOperands),
    Clz32(IntrinsicCallOperands),
    Atanh(IntrinsicCallOperands),
    Atan2(IntrinsicCallOperands),
    Round(IntrinsicCallOperands),
    Acosh(IntrinsicCallOperands),
    Abs(IntrinsicCallOperands),
    Sinh(IntrinsicCallOperands),
    Sin(IntrinsicCallOperands),
    Ceil(IntrinsicCallOperands),
    Tan(IntrinsicCallOperands),
    Trunc(IntrinsicCallOperands),
    Asinh(IntrinsicCallOperands),
    Log10(IntrinsicCallOperands),
    Asin(IntrinsicCallOperands),
    Random(IntrinsicCallOperands),
    Log1p(IntrinsicCallOperands),
    Sqrt(IntrinsicCallOperands),
    Atan(IntrinsicCallOperands),
    Cos(IntrinsicCallOperands),
    Tanh(IntrinsicCallOperands),
    Log(IntrinsicCallOperands),
    Floor(IntrinsicCallOperands),
    Cosh(IntrinsicCallOperands),
    Acos(IntrinsicCallOperands),
}

pub struct IntrinsicOperands<S: ExtractSource>(pub IntrinsicKind<S>);

impl<S: ExtractSource> ExtractBack<S> for IntrinsicOperands<S> {
    type Exception = Infallible;

    fn extract_back(cx: &mut S) -> Result<Self, Self::Exception> {
        let kind = IntrinsicOperation::from_repr(cx.fetch_u8()).unwrap();

        let operands = match kind {
            IntrinsicOperation::AddNumLR => IntrinsicKind::AddNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::SubNumLR => IntrinsicKind::SubNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::MulNumLR => IntrinsicKind::MulNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::DivNumLR => IntrinsicKind::DivNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::RemNumLR => IntrinsicKind::RemNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::PowNumLR => IntrinsicKind::PowNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::GtNumLR => IntrinsicKind::GtNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::GtNumLConstR => {
                IntrinsicKind::GtNumLConstR(cx.pop_stack_rooted(), extract_back_infallible(cx))
            }
            IntrinsicOperation::GtNumLConstR32 => {
                IntrinsicKind::GtNumLConstR32(cx.pop_stack_rooted(), extract_back_infallible(cx))
            }
            IntrinsicOperation::GeNumLR => IntrinsicKind::GeNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::GeNumLConstR => {
                IntrinsicKind::GeNumLConstR(cx.pop_stack_rooted(), extract_back_infallible(cx))
            }
            IntrinsicOperation::GeNumLConstR32 => {
                IntrinsicKind::GeNumLConstR32(cx.pop_stack_rooted(), extract_back_infallible(cx))
            }
            IntrinsicOperation::LtNumLR => IntrinsicKind::LtNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::LtNumLConstR => {
                IntrinsicKind::LtNumLConstR(cx.pop_stack_rooted(), extract_back_infallible(cx))
            }
            IntrinsicOperation::LtNumLConstR32 => {
                IntrinsicKind::LtNumLConstR32(cx.pop_stack_rooted(), extract_back_infallible(cx))
            }
            IntrinsicOperation::LeNumLR => IntrinsicKind::LeNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::LeNumLConstR => {
                IntrinsicKind::LeNumLConstR(cx.pop_stack_rooted(), extract_back_infallible(cx))
            }
            IntrinsicOperation::LeNumLConstR32 => {
                IntrinsicKind::LeNumLConstR32(cx.pop_stack_rooted(), extract_back_infallible(cx))
            }
            IntrinsicOperation::EqNumLR => IntrinsicKind::EqNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::NeNumLR => IntrinsicKind::NeNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::BitOrNumLR => IntrinsicKind::BitOrNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::BitXorNumLR => IntrinsicKind::BitXorNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::BitAndNumLR => IntrinsicKind::BitAndNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::BitShlNumLR => IntrinsicKind::BitShlNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::BitShrNumLR => IntrinsicKind::BitShrNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted()),
            IntrinsicOperation::BitUshrNumLR => {
                IntrinsicKind::BitUshrNumLR(cx.pop_stack_rooted(), cx.pop_stack_rooted())
            }
            IntrinsicOperation::PostfixIncLocalNum => IntrinsicKind::PostfixIncLocalNum(cx.pop_stack_rooted()),
            IntrinsicOperation::PostfixDecLocalNum => IntrinsicKind::PostfixDecLocalNum(cx.pop_stack_rooted()),
            IntrinsicOperation::PrefixIncLocalNum => IntrinsicKind::PrefixIncLocalNum(cx.pop_stack_rooted()),
            IntrinsicOperation::PrefixDecLocalNum => IntrinsicKind::PrefixDecLocalNum(cx.pop_stack_rooted()),
            IntrinsicOperation::Exp => IntrinsicKind::Exp(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Log2 => IntrinsicKind::Log2(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Expm1 => IntrinsicKind::Expm1(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Cbrt => IntrinsicKind::Cbrt(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Clz32 => IntrinsicKind::Clz32(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Atanh => IntrinsicKind::Atanh(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Atan2 => IntrinsicKind::Atan2(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Round => IntrinsicKind::Round(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Acosh => IntrinsicKind::Acosh(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Abs => IntrinsicKind::Abs(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Sinh => IntrinsicKind::Sinh(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Sin => IntrinsicKind::Sin(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Ceil => IntrinsicKind::Ceil(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Tan => IntrinsicKind::Tan(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Trunc => IntrinsicKind::Trunc(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Asinh => IntrinsicKind::Asinh(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Log10 => IntrinsicKind::Log10(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Asin => IntrinsicKind::Asin(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Random => IntrinsicKind::Random(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Log1p => IntrinsicKind::Log1p(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Sqrt => IntrinsicKind::Sqrt(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Atan => IntrinsicKind::Atan(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Cos => IntrinsicKind::Cos(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Tanh => IntrinsicKind::Tanh(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Log => IntrinsicKind::Log(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Floor => IntrinsicKind::Floor(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Cosh => IntrinsicKind::Cosh(IntrinsicCallOperands { argc: cx.fetch_u8() }),
            IntrinsicOperation::Acos => IntrinsicKind::Acos(IntrinsicCallOperands { argc: cx.fetch_u8() }),
        };

        Ok(Self(operands))
    }
}
