use core::fmt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Write;
use std::io::Read;

use crate::compiler::instruction;

use super::constant::Constant;
use super::CompileResult;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum Unit {
    Main,
    Function(Cow<'static, str>),
}

struct Output {
    units: HashMap<Unit, String>,
}

impl Unit {
    pub fn name(&self) -> &str {
        match self {
            Unit::Main => "{{unnamed function}}",
            Unit::Function(name) => name.as_ref(),
        }
    }
}

impl Output {
    pub fn new() -> Self {
        Output {
            units: HashMap::new(),
        }
    }

    pub fn write_instruction<D: Display>(&mut self, unit: Unit, instruction: &str, args: &[D]) {
        let unit = self
            .units
            .entry(unit.clone())
            .or_insert_with(|| format!("{}\n", unit.name()));

        let _ = write!(unit, "    {: <8}", instruction);

        for arg in args {
            let _ = write!(unit, " {}", arg);
        }

        unit.push('\n');
    }

    pub fn finish(self) -> String {
        let mut output = String::new();

        for code in self.units.values() {
            output.push_str(&code);
            output.push_str("\n");
        }

        output
    }
}

struct Reader<R: Read>(R, usize);

impl<R: Read> Reader<R> {
    pub fn read_bytes<const N: usize>(&mut self) -> Option<[u8; N]> {
        let mut buf = [0; N];
        self.0.read_exact(&mut buf).ok()?;
        Some(buf)
    }

    pub fn read(&mut self) -> Option<u8> {
        self.read_bytes::<1>().map(|[b]| b)
    }

    pub fn read_u16_ne(&mut self) -> Option<u16> {
        self.read_bytes().map(u16::from_ne_bytes)
    }
}

#[derive(Debug)]
pub enum DecompileError {
    AbruptEof,
    UnknownInstruction(u8),
}

enum StackValue {
    Number(f64),
    String(String),
    Identifier(String),
    Boolean(bool),
    Null,
    Undefined,
}
impl From<Constant> for StackValue {
    fn from(constant: Constant) -> Self {
        match constant {
            Constant::Number(n) => StackValue::Number(n),
            Constant::String(s) => StackValue::String(s),
            Constant::Identifier(i) => StackValue::Identifier(i),
            Constant::Boolean(b) => StackValue::Boolean(b),
            Constant::Null => StackValue::Null,
            Constant::Undefined => StackValue::Undefined,
        }
    }
}
impl fmt::Display for StackValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackValue::Number(n) => write!(f, "{}", n),
            StackValue::String(s) => write!(f, "{}", s),
            StackValue::Identifier(s) => write!(f, "{}", s),
            StackValue::Boolean(b) => write!(f, "{}", b),
            StackValue::Null => write!(f, "null"),
            StackValue::Undefined => write!(f, "undefined"),
        }
    }
}

struct StackId(usize);

impl fmt::Display for StackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[sp+{}]", self.0)
    }
}

pub fn decompile(
    CompileResult { cp, instructions }: CompileResult,
) -> Result<String, DecompileError> {
    let mut reader = Reader(instructions.as_slice(), 0);
    let mut stack = 0;
    let mut output = Output::new();

    fn handle_arithmetic(
        stack: &mut Vec<StackValue>,
        output: &mut Output,
        instruction: &str,
    ) -> Result<(), DecompileError> {
        let (a, b) = stack
            .pop()
            .zip(stack.pop())
            .ok_or(DecompileError::AbruptEof)?;

        output.write_instruction(Unit::Main, instruction, &[a, b]);
        Ok(())
    }

    loop {
        let instr = reader.read().ok_or(DecompileError::AbruptEof)?;

        match instr {
            instruction::ADD => {
                stack -= 2;
                output.write_instruction(
                    Unit::Main,
                    "ADD",
                    &[StackId(stack + 1), StackId(stack + 2)],
                );
                stack += 1;
            }
            instruction::CONSTANT | instruction::CONSTANTW => {
                let constant = if instr == instruction::CONSTANT {
                    let id = reader.read().ok_or(DecompileError::AbruptEof)?;
                    cp[id as usize].clone()
                } else {
                    let id = reader.read_u16_ne().ok_or(DecompileError::AbruptEof)?;
                    cp[id as usize].clone()
                };

                let constant = StackValue::from(constant);

                stack += 1;
                let args: &[&dyn Display] = &[&StackId(stack), &constant];
                output.write_instruction(Unit::Main, "CONSTANT", args);
            }
            instruction::POP => {
                stack -= 1;
            }
            instruction::RET => {
                output.write_instruction(Unit::Main, "RET", &[StackId(stack)]);
                break;
            }
            _ => return Err(DecompileError::UnknownInstruction(instr)),
        }
    }

    Ok(output.finish())
}
