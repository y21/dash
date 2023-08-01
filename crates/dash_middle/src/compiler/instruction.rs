#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum_macros::FromRepr;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
    Pop,
    LdLocal,
    LdLocalW,
    LdGlobal,
    LdGlobalW,
    Constant,
    ConstantW,
    Pos,
    Neg,
    TypeOf,
    BitNot,
    Not,
    StoreLocal,
    StoreLocalW,
    StoreGlobal,
    StoreGlobalW,
    Ret,
    Call,
    JmpFalseP,
    Jmp,
    StaticPropAccess,
    StaticPropAccessW,
    DynamicPropAccess,
    ArrayLit,
    ArrayLitW,
    ObjLit,
    ObjLitW,
    This,
    StaticPropAssign,
    DynamicPropAssign,
    /// Loads an external variable
    LdLocalExt,
    LdLocalExtW,
    /// Stores a value into an external variable
    StoreLocalExt,
    StoreLocalExtW,
    StrictEq,
    StrictNe,
    Try,
    TryEnd,
    Throw,
    Yield,
    JmpFalseNP,
    JmpTrueP,
    JmpTrueNP,
    JmpNullishP,
    JmpNullishNP,
    JmpUndefinedNP,
    JmpUndefinedP,
    BitOr,
    BitXor,
    BitAnd,
    BitShl,
    BitShr,
    BitUshr,
    ObjIn,
    InstanceOf,
    ImportDyn,
    ImportStatic,
    ExportDefault,
    ExportNamed,
    Debugger,
    Global,
    Super,
    Undef,
    Await,
    Nan,
    Infinity,
    IntrinsicOp,
    CallSymbolIterator,
    CallForInIterator,
    DeletePropertyStatic,
    DeletePropertyDynamic,
    Switch,
    ObjDestruct,
    ArrayDestruct,
    // Nop exists solely for the sake of benchmarking the raw throughput of the VM dispatch loop
    Nop,
}

// Some instruction opcodes have a separate u8 constant to be used in for example match guards,
// where `Instruction::Pop as u8` isn't allowed
pub const POP: u8 = Instruction::Pop as u8;
pub const RET: u8 = Instruction::Ret as u8;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AssignKind {
    Assignment,
    AddAssignment,
    SubAssignment,
    MulAssignment,
    DivAssignment,
    RemAssignment,
    PowAssignment,
    ShlAssignment,
    ShrAssignment,
    UshrAssignment,
    BitAndAssignment,
    BitOrAssignment,
    BitXorAssignment,
    PrefixIncrement,
    PostfixIncrement,
    PrefixDecrement,
    PostfixDecrement,
}

/// Intrinsic operations, i.e. operations known by the compiler. These can be
/// specialized operations, such as the `+` operator on two numbers.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IntrinsicOperation {
    /// Addition, where the left and right side is a number
    AddNumLR,
    /// Subtraction, where the left and right side is a number
    SubNumLR,
    /// Multiplication, where the left and right side is a number
    MulNumLR,
    /// Division, where the left and right side is a number
    DivNumLR,
    /// Remainder, where the left and right side is a number
    RemNumLR,
    /// Power, where the left and right side is a number
    PowNumLR,
    /// Greater than, where the left and right side is a number
    GtNumLR,
    /// Greater than, where the left side is a number and right side is a constant number (u8)
    GtNumLConstR,
    /// Greater than, where the left side is a number and right side is a constant number (u32)
    GtNumLConstR32,
    /// Greater than or equal, where the left and right side is a number
    GeNumLR,
    /// Greater than or equal, where the left side is a number and right side is a constant number (u8)
    GeNumLConstR,
    /// Greater than or equal, where the left side is a number and right side is a constant number (u32)
    GeNumLConstR32,
    /// Less than, where the left and right side is a number
    LtNumLR,
    /// Less than, where the left side is a number and right side is a constant number (u8)
    LtNumLConstR,
    /// Less than, where the left side is a number and right side is a constant number (u32)
    LtNumLConstR32,
    /// Less than or equal, where the left and right side is a number
    LeNumLR,
    /// Less than or equal, where the left side is a number and right side is a constant number (u8)
    LeNumLConstR,
    /// Less than or equal, where the left side is a number and right side is a constant number (u32)
    LeNumLConstR32,
    /// Equal, where the left and right side is a number
    EqNumLR,
    /// Not equal, where the left and right side is a number
    NeNumLR,
    /// Bitwise or, where the left and right side is a number
    BitOrNumLR,
    /// Bitwise xor, where the left and right side is a number
    BitXorNumLR,
    /// Bitwise and, where the left and right side is a number
    BitAndNumLR,
    /// Bitwise shift left, where the left and right side is a number
    BitShlNumLR,
    /// Bitwise shift right, where the left and right side is a number
    BitShrNumLR,
    /// Bitwise unsigned shift right, where the left and right side is a number
    BitUshrNumLR,
    /// Postfix increment, where the left side is an identifier that refers to a variable of type number
    PostfixIncLocalNum,
    /// Postfix decrement, where the left side is an identifier that refers to a variable of type number
    PostfixDecLocalNum,
    /// Prefix increment, where the left side is an identifier that refers to a variable of type number
    PrefixIncLocalNum,
    /// Prefix decrement, where the left side is an identifier that refers to a variable of type number
    PrefixDecLocalNum,
    Exp,
    Log2,
    Expm1,
    Cbrt,
    Clz32,
    Atanh,
    Atan2,
    Round,
    Acosh,
    Abs,
    Sinh,
    Sin,
    Ceil,
    Tan,
    Trunc,
    Asinh,
    Log10,
    Asin,
    Random,
    Log1p,
    Sqrt,
    Atan,
    Cos,
    Tanh,
    // PI
    Log,
    Floor,
    Cosh,
    Acos,
}
