use crate::{gc::Handle, parser::token::TokenType};

use super::{
    value::{
        function::{FunctionKind, UserFunction},
        Value, ValueKind,
    },
    VM,
};

/// A VM opcode, used in bytecode to denote the type of work it should do
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opcode {
    /// Pushes a constant on the stack
    Constant,
    /// End of file: an abrupt failure, or the end of a script
    Eof,
    /// Sets a local without a value
    SetLocalNoValue,
    /// Sets the value of a local variable
    SetLocal,
    /// Sets the value of an upvalue
    SetUpvalue,
    /// Whether a local is an upvalue
    UpvalueLocal,
    /// Whether a local is not an upvalue
    // TODO: we might be able to remove this as it's not used in the VM
    UpvalueNonLocal,
    /// Gets a local and pushes it on the stack
    GetLocal,
    /// Gets the current `this` value and pushes it on the stack
    GetThis,
    /// Gets the current `super` value and pushes it on the stack
    GetSuper,
    /// Gets the global namespace and pushes it on the stack
    GetGlobalThis,
    /// Gets an upvalue and pushes it on the stack
    GetUpvalue,
    /// Creates a global without a value
    SetGlobalNoValue,
    /// Creates a global variable with a value
    SetGlobal,
    /// Gets a global and pushes it on the stack
    GetGlobal,
    /// Performs bitwise and
    BitwiseAnd,
    /// Performs bitwise or
    BitwiseOr,
    /// Performs bitwise xor
    BitwiseXor,
    /// Performs the addition assignment operation
    AdditionAssignment,
    /// Performs the subtraction assignment operation
    SubtractionAssignment,
    /// Performs the multiplication assignment operation
    MultiplicationAssignment,
    /// Performs the division assignment operation
    DivisionAssignment,
    /// Performs the remainder assignment operation
    RemainderAssignment,
    /// Performs the exponentiation assignment operation
    ExponentiationAssignment,
    /// Performs the left shift assignment operation
    LeftShiftAssignment,
    /// Performs the right shift assignment operation
    RightShiftAssignment,
    /// Performs the unsigned right shift assignment operation
    UnsignedRightShiftAssignment,
    /// Performs the bitwise and assignment operation
    BitwiseAndAssignment,
    /// Performs the bitwise or assignment operation
    BitwiseOrAssignment,
    /// Performs the bitwise xor assignment operation
    BitwiseXorAssignment,
    /// Performs the logical and assignment operation
    LogicalAndAssignment,
    /// Performs the logical or assignment operation
    LogicalOrAssignment,
    /// Performs the logical nullish assignment operation
    LogicalNullishAssignment,
    /// Performs an assignment operation
    Assignment,
    /// Performs addition
    Add,
    /// Performs subtraction
    Sub,
    /// Performs multiplication
    Mul,
    /// Performs division
    Div,
    /// Performs remainder
    Rem,
    /// Performs exponentiation
    Exponentiation,
    /// Performs left shift
    LeftShift,
    /// Performs right shift
    RightShift,
    /// Performs unsigned right shift
    UnsignedRightShift,
    /// Performs
    Positive,
    /// Negates a number
    Negate,
    /// Performs logical not
    LogicalNot,
    /// Performs bitwise not
    BitwiseNot,
    /// Jumps over a chunk of code
    ShortJmp,
    /// Jumps over a chunk of code if a condition is false
    ShortJmpIfFalse,
    /// Jumps over a chunk of code if a condition is true
    ShortJmpIfTrue,
    /// Jumps over a chunk of code if a condition is nullish
    ShortJmpIfNullish,
    /// Jumps back to a chunk of code
    BackJmp,
    /// Discards a value on the stack
    Pop,
    /// Same as `Pop`, but allows the compiler to elide this pop instruction if it's the last one
    ///
    /// In particular, constructs like for and if generate jump instructions that expect instructions
    /// not to be removed. If that is the case, `Pop` should be used.
    PopElide,
    /// Pops an unwind handler
    PopUnwindHandler,
    /// Calls a function
    FunctionCall,
    /// Calls a function as a constructor
    ConstructorCall,
    /// Returns from the current execution frame
    Return,
    /// Returns from the current JavaScript module
    ReturnModule,
    /// No-op, do nothing
    // TODO: change to InvalidInstruction or similar
    Nop,
    /// Performs less (<)
    Less,
    /// Performs less equal
    LessEqual,
    /// Performs greater (>)
    Greater,
    /// Performs greater equal
    GreaterEqual,
    /// Looks up a property on an object by a literal
    StaticPropertyAccess,
    /// Looks up a property using a computed value as key
    ComputedPropertyAccess,
    /// Performs a `typeof` operation
    Typeof,
    /// A Closure
    Closure,
    /// Performs the equality check operation
    Equality,
    /// Performs the inequality check operation
    Inequality,
    /// Performs the strict equality check operation
    StrictEquality,
    /// Performs the strict inequality check operation
    StrictInequality,
    /// Performs postfix increment (x++)
    PostfixIncrement,
    /// Performs postfix decrement (x--)
    PostfixDecrement,
    /// Replaces the last value on the stack with an undefined JavaScript value
    Void,
    /// Array literal, used to create and initialize an array
    ArrayLiteral,
    /// Object literal, used to create and initialize an object
    ObjectLiteral,
    /// Try/Catch block
    Try,
    /// Throw statement, throws an error
    Throw,
    /// Performs `continue` in a loop
    Continue,
    /// Performs `break` in a loop
    Break,
    /// Indicates the start of a loop
    LoopStart,
    /// Indicates the end of a loop
    LoopEnd,
    /// Similar to FunctionCall, but evaluates a module
    EvaluateModule,
    /// Exports a default value
    ExportDefault,
    /// Performs the abstract `ToPrimitive` operation
    ToPrimitive,
    /// Invokes the debugger, if present
    Debugger,
}

impl From<TokenType> for Opcode {
    fn from(tt: TokenType) -> Self {
        match tt {
            TokenType::Plus => Self::Add,
            TokenType::Minus => Self::Sub,
            TokenType::Star => Self::Mul,
            TokenType::Slash => Self::Div,
            TokenType::Remainder => Self::Rem,
            TokenType::BitwiseAnd => Self::BitwiseAnd,
            TokenType::BitwiseOr => Self::BitwiseOr,
            TokenType::BitwiseXor => Self::BitwiseXor,
            TokenType::LeftShift => Self::LeftShift,
            TokenType::RightShift => Self::RightShift,
            TokenType::Exponentiation => Self::Exponentiation,
            TokenType::AdditionAssignment => Self::AdditionAssignment,
            TokenType::SubtractionAssignment => Self::SubtractionAssignment,
            TokenType::MultiplicationAssignment => Self::MultiplicationAssignment,
            TokenType::DivisionAssignment => Self::DivisionAssignment,
            TokenType::RemainderAssignment => Self::RemainderAssignment,
            TokenType::ExponentiationAssignment => Self::ExponentiationAssignment,
            TokenType::LeftShiftAssignment => Self::LeftShiftAssignment,
            TokenType::RightShiftAssignment => Self::RightShiftAssignment,
            TokenType::UnsignedRightShiftAssignment => Self::UnsignedRightShiftAssignment,
            TokenType::BitwiseAndAssignment => Self::BitwiseAndAssignment,
            TokenType::BitwiseOrAssignment => Self::BitwiseOrAssignment,
            TokenType::BitwiseXorAssignment => Self::BitwiseXorAssignment,
            TokenType::LogicalAndAssignment => Self::LogicalAndAssignment,
            TokenType::LogicalOrAssignment => Self::LogicalOrAssignment,
            TokenType::LogicalNullishAssignment => Self::LogicalNullishAssignment,
            TokenType::PrefixIncrement => Self::AdditionAssignment,
            TokenType::PrefixDecrement => Self::SubtractionAssignment,
            TokenType::PostfixIncrement | TokenType::Increment => Self::PostfixIncrement,
            TokenType::PostfixDecrement | TokenType::Decrement => Self::PostfixDecrement,
            TokenType::Assignment => Self::Assignment,
            TokenType::Less => Self::Less,
            TokenType::LessEqual => Self::LessEqual,
            TokenType::Greater => Self::Greater,
            TokenType::GreaterEqual => Self::GreaterEqual,
            TokenType::Equality => Self::Equality,
            TokenType::Inequality => Self::Inequality,
            TokenType::StrictEquality => Self::StrictEquality,
            TokenType::StrictInequality => Self::StrictInequality,
            _ => unimplemented!("{:?}", tt),
        }
    }
}

/// A constant
#[derive(Debug, Clone)]
pub enum Constant {
    /// A JavaScript value
    JsValue(Handle<Value>),
    /// An identifier
    Identifier(String),
    /// An index
    Index(usize),
    /// A function
    Function(FunctionKind),
}

impl Constant {
    /// Returns self as an owned JavaScript value, if it one
    pub fn into_value(self) -> Option<Handle<Value>> {
        match self {
            Self::JsValue(v) => Some(v),
            _ => None,
        }
    }

    /// Returns self as an owned JavaScript value
    pub fn try_into_value(self) -> Option<Handle<Value>> {
        match self {
            Self::JsValue(v) => Some(v),
            _ => None,
        }
    }

    /// Returns self as an owned identifier, if it is one
    pub fn into_ident(self) -> Option<String> {
        match self {
            Self::Identifier(ident) => Some(ident),
            _ => None,
        }
    }

    /// Returns self as an owned index, if it is one
    pub fn into_index(self) -> Option<usize> {
        match self {
            Self::Index(idx) => Some(idx),
            _ => None,
        }
    }

    /// Returns self as an index, if it is one
    pub fn into_function(self) -> Option<FunctionKind> {
        match self {
            Self::Function(fun) => Some(fun),
            _ => None,
        }
    }

    /// Returns self as an index, if it is one
    pub fn as_index(&self) -> Option<usize> {
        match self {
            Self::Index(idx) => Some(*idx),
            _ => None,
        }
    }
}

/// A bytecode instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    /// An opcode
    Op(Opcode),
    /// An operand carrying a value
    Operand(Constant),
}

impl Instruction {
    /// Returns self as an opcode and consumes self
    pub fn into_op(self) -> Opcode {
        match self {
            Self::Op(o) => o,
            _ => unreachable!(),
        }
    }

    /// Returns self as an opcode
    pub fn as_op(&self) -> Opcode {
        match self {
            Self::Op(o) => *o,
            _ => unreachable!(),
        }
    }

    /// Returns self as a constant and consumes self
    pub fn into_operand(self) -> Constant {
        match self {
            Self::Operand(o) => o,
            _ => unreachable!(),
        }
    }
}
