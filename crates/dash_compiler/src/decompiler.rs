use core::fmt;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Write;
use std::io::Read;
use std::rc::Rc;

use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::constant::Function;
use dash_middle::compiler::instruction as inst;

use super::CompileResult;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum Unit {
    Main,
    // Function(Cow<'static, str>),
}

struct Output {
    units: HashMap<Unit, String>,
}

impl Unit {
    pub fn name(&self) -> &str {
        match self {
            Unit::Main => "{{unnamed function}}",
            // Unit::Function(name) => name.as_ref(),
        }
    }
}

impl Output {
    pub fn new() -> Self {
        Output { units: HashMap::new() }
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

    pub fn read_i16_ne(&mut self) -> Option<i16> {
        self.read_bytes().map(i16::from_ne_bytes)
    }
}

#[derive(Debug)]
pub enum DecompileError {
    AbruptEof,
    UnknownInstruction(u8),
}

enum StackValue {
    Number(f64),
    String(Rc<str>),
    Identifier(Rc<str>),
    Boolean(bool),
    Function(Rc<Function>),
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
            Constant::Function(f) => StackValue::Function(f),
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
            StackValue::Function(n) => {
                write!(f, "function {}", n.name.as_deref().unwrap_or("{{unnamed}}"))
            }
        }
    }
}

struct StackId(usize);

impl fmt::Display for StackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[sp+{}]", self.0)
    }
}

fn handle_arithmetic(stack: &mut usize, output: &mut Output, instruction: &str) {
    *stack -= 2;
    output.write_instruction(Unit::Main, instruction, &[StackId(*stack + 1), StackId(*stack + 2)]);
    *stack += 1;
}

fn read_wide<'a, R: Read>(
    actual: u8,
    thin: (&'a str, u8),
    wide: (&'a str, u8),
    reader: &mut Reader<R>,
) -> Result<(&'a str, u16), DecompileError> {
    if actual == thin.1 {
        let id = reader.read().ok_or(DecompileError::AbruptEof)?;
        Ok((thin.0, id as u16))
    } else {
        let id = reader.read_u16_ne().ok_or(DecompileError::AbruptEof)?;
        Ok((wide.0, id))
    }
}

// fn read_wide_signed<'a, R: Read>(
//     actual: u8,
//     thin: (&'a str, u8),
//     wide: (&'a str, u8),
//     reader: &mut Reader<R>,
// ) -> Result<(&'a str, i16), DecompileError> {
//     if actual == thin.1 {
//         let id = reader.read().ok_or(DecompileError::AbruptEof)?;
//         Ok((thin.0, id as i8 as i16))
//     } else {
//         let id = reader.read_u16_ne().ok_or(DecompileError::AbruptEof)?;
//         Ok((wide.0, id as i16))
//     }
// }

pub fn decompile(CompileResult { cp, instructions, .. }: CompileResult) -> Result<String, DecompileError> {
    let mut reader = Reader(instructions.as_slice(), 0);
    let mut stack = 0;
    let mut output = Output::new();

    loop {
        let instr = reader.read().ok_or(DecompileError::AbruptEof)?;

        match instr {
            inst::ADD => handle_arithmetic(&mut stack, &mut output, "ADD"),
            inst::SUB => handle_arithmetic(&mut stack, &mut output, "SUB"),
            inst::MUL => handle_arithmetic(&mut stack, &mut output, "MUL"),
            inst::DIV => handle_arithmetic(&mut stack, &mut output, "DIV"),
            inst::REM => handle_arithmetic(&mut stack, &mut output, "REM"),
            inst::POW => handle_arithmetic(&mut stack, &mut output, "POW"),
            inst::GT => handle_arithmetic(&mut stack, &mut output, "GT"),
            inst::GE => handle_arithmetic(&mut stack, &mut output, "GE"),
            inst::LT => handle_arithmetic(&mut stack, &mut output, "LT"),
            inst::LE => handle_arithmetic(&mut stack, &mut output, "LE"),
            inst::EQ => handle_arithmetic(&mut stack, &mut output, "EQ"),
            inst::NE => handle_arithmetic(&mut stack, &mut output, "NE"),
            inst::POP => {
                output.write_instruction::<u8>(Unit::Main, "POP", &[]);
                stack -= 1;
            }
            inst::REVSTCK => {
                let n = reader.read().ok_or(DecompileError::AbruptEof)?;
                output.write_instruction(Unit::Main, "REVSTCK", &[n]);
            }
            inst::CONSTANT | inst::CONSTANTW => {
                let (name, id) = read_wide(
                    instr,
                    ("CONSTANT", inst::CONSTANT),
                    ("CONSTANTW", inst::CONSTANTW),
                    &mut reader,
                )?;
                let constant = StackValue::from(cp[id as usize].clone());

                let args: &[&dyn Display] = &[&StackId(stack), &constant];
                output.write_instruction(Unit::Main, name, args);
                stack += 1;
            }
            inst::LDLOCAL | inst::LDLOCALW => {
                let (name, id) = read_wide(
                    instr,
                    ("LDLOCAL", inst::LDLOCAL),
                    ("LDLOCALW", inst::LDLOCALW),
                    &mut reader,
                )?;
                stack += 1;
                output.write_instruction(Unit::Main, name, &[StackId(id.into())]);
            }
            inst::JMP => {
                let id = reader.read_i16_ne().ok_or(DecompileError::AbruptEof)?;
                output.write_instruction(Unit::Main, "JMP", &[id]);
            }
            inst::JMPFALSEP => {
                let id = reader.read_i16_ne().ok_or(DecompileError::AbruptEof)?;
                output.write_instruction(Unit::Main, "JMPFALSEP", &[id]);
            }
            inst::LDGLOBAL | inst::LDGLOBALW => {
                let (name, id) = read_wide(
                    instr,
                    ("LDGLOBAL", inst::LDGLOBAL),
                    ("LDGLOBALW", inst::LDGLOBALW),
                    &mut reader,
                )?;
                let constant = StackValue::from(cp[id as usize].clone());

                let args: &[&dyn Display] = &[&StackId(stack), &constant];
                output.write_instruction(Unit::Main, name, args);
                stack += 1;
            }
            inst::STORELOCAL | inst::STORELOCALW => {
                let (name, id) = read_wide(
                    instr,
                    ("STORELOCAL", inst::STORELOCAL),
                    ("STORELOCALW", inst::STORELOCALW),
                    &mut reader,
                )?;

                let args: &[&dyn Display] = &[&StackId(id.into())];
                output.write_instruction(Unit::Main, name, args);
                stack += 1;
            }
            inst::CALL => {
                let argc = reader.read().ok_or(DecompileError::AbruptEof)?;
                let is_constructor = reader.read().ok_or(DecompileError::AbruptEof)?;

                stack -= argc as usize + 1;
                let args: &[&dyn Display] = &[
                    &StackId(stack),
                    &format!("argc={argc}"),
                    &format!("is_constructor={is_constructor}"),
                ];

                output.write_instruction(Unit::Main, "CALL", args);
                stack += 1;
            }
            inst::STATICPROPACCESS | inst::STATICPROPACCESSW => {
                let (name, id) = read_wide(
                    instr,
                    ("STATICPROPACCESS", inst::STATICPROPACCESS),
                    ("STATICPROPACCESSW", inst::STATICPROPACCESSW),
                    &mut reader,
                )?;
                let constant = StackValue::from(cp[id as usize].clone());

                let args: &[&dyn Display] = &[&StackId(stack), &constant];
                output.write_instruction(Unit::Main, name, args);
                stack += 1;
            }
            inst::RET => {
                output.write_instruction(Unit::Main, "RET", &[StackId(stack)]);
                break;
            }
            _ => return Err(DecompileError::UnknownInstruction(instr)),
        }
    }

    Ok(output.finish())
}

#[test]
fn decompile_test() {
    use crate::FunctionCompiler;

    let c = FunctionCompiler::compile_str(
        r#"
let n = 1;
if (n == 1) {
    if (n == 2) {}
}
    "#,
        dash_optimizer::OptLevel::None,
    )
    .unwrap();
    let s = decompile(c).unwrap();
    println!("{s}");
}
