use crate::vm::{
    instruction::{Constant, Instruction},
    value::{
        array::Array,
        function::{FunctionKind, Module, UserFunction},
        object::Object,
        Value, ValueKind,
    },
};

use super::serialize::Serialize;

/// A constant that defines the version of the snapshot format.
/// Whenever a breaking change is made, this constant must be incremented
pub const SNAPSHOT_VERSION: u8 = 1;

// TODO: use u64 instead of usize!

/// Serializes bytecode and constants into a vector of bytes that can be written to a file
/// and later read back in.
pub fn serialize(bytecode: Vec<Instruction>, constants: Vec<Constant>) -> Vec<u8> {
    let mut result = vec![SNAPSHOT_VERSION];

    result.extend(bytecode.len().to_le_bytes());
    result.extend(bytecode.serialize());

    result.extend(constants.serialize());

    result
}

#[repr(u8)]
enum ConstantDiscriminant {
    Identifier = 0,
    Index = 1,
    Function = 2,
    Value = 3,
}

impl Serialize for Constant {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        match &self {
            Self::Identifier(ident) => {
                // Enum discriminant
                data.push(ConstantDiscriminant::Identifier as u8);

                // Write identifier length
                data.extend(ident.serialize());

                // Write identifier in UTF8
                data.extend(ident.as_bytes());
            }
            Self::Index(idx) => {
                // Enum discriminant
                data.push(ConstantDiscriminant::Index as u8);

                // Write index
                data.extend(idx.to_le_bytes());
            }
            Self::Function(func) => {
                // Enum discriminant
                data.push(ConstantDiscriminant::Function as u8);

                // Write serialized function kind
                data.extend(func.serialize());
            }
            Self::JsValue(handle) => {
                // Enum discriminant
                data.push(ConstantDiscriminant::Value as u8);

                // TODO: get rid of unsafe here
                let value = unsafe { handle.borrow_unbounded() };

                data.extend(value.serialize());
            }
        }

        data
    }
}

#[repr(u8)]
enum FunctionKindDiscriminant {
    Closure = 0,
    Module = 1,
    User = 2,
}

impl Serialize for FunctionKind {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        match &self {
            Self::Closure(c) => {
                // Enum discriminant
                data.push(FunctionKindDiscriminant::Closure as u8);
                data.extend(c.func.serialize());
            }
            Self::Module(m) => {
                // Enum discriminant
                data.push(FunctionKindDiscriminant::Module as u8);
                data.extend(m.serialize());
            }
            Self::Native(_) => unreachable!("not emitted by the compiler"),
            Self::User(u) => {
                // Enum discriminant
                data.push(FunctionKindDiscriminant::User as u8);
                data.extend(u.serialize());
            }
        }

        data
    }
}

impl Serialize for str {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend(self.len().to_le_bytes());
        data.extend(self.as_bytes());

        data
    }
}

impl Serialize for UserFunction {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Parameter count
        data.extend(self.params.to_le_bytes());

        // Type
        data.push(self.ty as u8);

        // Buffer
        data.extend(self.buffer.len().to_le_bytes());
        data.extend(self.buffer.serialize());

        // Constants
        data.extend(self.constants.serialize());

        // Has string
        data.push(self.name.is_some() as u8);

        // Write name if present
        if let Some(name) = &self.name {
            data.extend(name.serialize());
        }

        // Upvalues
        data.extend(self.upvalues.to_le_bytes());

        data
    }
}

impl Serialize for [Instruction] {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        for instruction in self {
            data.push(instruction.as_operand());
        }

        data
    }
}

impl Serialize for [Constant] {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        for constant in self {
            data.extend(constant.serialize());
        }

        data
    }
}

impl Serialize for Module {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Buffer
        data.push(self.buffer.is_some() as u8);
        if let Some(buffer) = &self.buffer {
            data.extend(buffer.len().to_le_bytes());
            data.extend(buffer.serialize());
        }

        // Constants
        data.extend(self.constants.serialize());

        data
    }
}

impl Serialize for Value {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Write value kind
        data.extend(self.kind.serialize());

        // Write fields
        data.extend(self.fields.len().to_le_bytes());

        for (key, handle) in &self.fields {
            // Write key
            data.extend(key.serialize());

            // Write value
            let value = unsafe { handle.borrow_unbounded() };
            data.extend(value.serialize());
        }

        data
    }
}

impl Serialize for ValueKind {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        match self {
            Self::Bool(b) => {
                data.push(ValueKindDiscriminant::Bool as u8);
                data.push(*b as u8);
            }
            Self::Number(n) => {
                data.push(ValueKindDiscriminant::Number as u8);
                data.extend(n.to_le_bytes());
            }
            Self::Object(o) => {
                data.push(ValueKindDiscriminant::Object as u8);
                data.extend(o.serialize());
            }
            Self::Undefined => {
                data.push(ValueKindDiscriminant::Undefined as u8);
            }
            Self::Null => {
                data.push(ValueKindDiscriminant::Null as u8);
            }
        }

        data
    }
}

#[repr(u8)]
enum ValueKindDiscriminant {
    Number,
    Bool,
    Object,
    Undefined,
    Null,
}

#[repr(u8)]
enum ObjectDiscriminant {
    String,
    Function,
    Array,
    Any,
}

impl Serialize for Object {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        match self {
            Self::String(s) => {
                data.push(ObjectDiscriminant::String as u8);
                data.extend(s.serialize());
            }
            Self::Function(f) => {
                data.push(ObjectDiscriminant::Function as u8);
                data.extend(f.serialize());
            }
            Self::Array(a) => {
                data.push(ObjectDiscriminant::Array as u8);
                data.extend(a.serialize());
            }
            Self::Any(_) => {
                data.push(ObjectDiscriminant::Any as u8);
            }
            _ => unreachable!("not emitted by the compiler"),
        }

        data
    }
}

impl Serialize for Array {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Write length
        data.extend(self.elements.len().to_le_bytes());

        // Write elements
        for handle in &self.elements {
            let value = unsafe { handle.borrow_unbounded() };
            data.extend(value.serialize());
        }

        data
    }
}
