use core::fmt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Write;
use std::io::Read;
use std::rc::Rc;

use super::constant::Constant;
use super::instruction::*;
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
    String(String),
    Identifier(Rc<str>),
    Boolean(bool),
    Function(Option<String>),
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
            Constant::Function(f) => StackValue::Function(f.name),
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
                write!(f, "function {}", n.as_deref().unwrap_or("{{unnamed}}"))
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

fn read_wide_signed<'a, R: Read>(
    actual: u8,
    thin: (&'a str, u8),
    wide: (&'a str, u8),
    reader: &mut Reader<R>,
) -> Result<(&'a str, i16), DecompileError> {
    if actual == thin.1 {
        let id = reader.read().ok_or(DecompileError::AbruptEof)?;
        Ok((thin.0, id as i8 as i16))
    } else {
        let id = reader.read_u16_ne().ok_or(DecompileError::AbruptEof)?;
        Ok((wide.0, id as i16))
    }
}

pub fn decompile(CompileResult { cp, instructions, .. }: CompileResult) -> Result<String, DecompileError> {
    let mut reader = Reader(instructions.as_slice(), 0);
    let mut stack = 0;
    let mut output = Output::new();

    loop {
        let instr = reader.read().ok_or(DecompileError::AbruptEof)?;

        match instr {
            ADD => handle_arithmetic(&mut stack, &mut output, "ADD"),
            SUB => handle_arithmetic(&mut stack, &mut output, "SUB"),
            MUL => handle_arithmetic(&mut stack, &mut output, "MUL"),
            DIV => handle_arithmetic(&mut stack, &mut output, "DIV"),
            REM => handle_arithmetic(&mut stack, &mut output, "REM"),
            POW => handle_arithmetic(&mut stack, &mut output, "POW"),
            GT => handle_arithmetic(&mut stack, &mut output, "GT"),
            GE => handle_arithmetic(&mut stack, &mut output, "GE"),
            LT => handle_arithmetic(&mut stack, &mut output, "LT"),
            LE => handle_arithmetic(&mut stack, &mut output, "LE"),
            EQ => handle_arithmetic(&mut stack, &mut output, "EQ"),
            NE => handle_arithmetic(&mut stack, &mut output, "NE"),
            POP => {
                output.write_instruction::<u8>(Unit::Main, "POP", &[]);
                stack -= 1;
            }
            CONSTANT | CONSTANTW => {
                let (name, id) = read_wide(instr, ("CONSTANT", CONSTANT), ("CONSTANTW", CONSTANTW), &mut reader)?;
                let constant = StackValue::from(cp[id as usize].clone());

                let args: &[&dyn Display] = &[&StackId(stack), &constant];
                output.write_instruction(Unit::Main, name, args);
                stack += 1;
            }
            LDLOCAL | LDLOCALW => {
                let (name, id) = read_wide(instr, ("LDLOCAL", LDLOCAL), ("LDLOCALW", LDLOCALW), &mut reader)?;
                stack += 1;
                output.write_instruction(Unit::Main, name, &[StackId(id.into())]);
            }
            JMP => {
                let id = reader.read_i16_ne().ok_or(DecompileError::AbruptEof)?;
                output.write_instruction(Unit::Main, "JMP", &[id]);
            }
            JMPFALSEP => {
                let id = reader.read_i16_ne().ok_or(DecompileError::AbruptEof)?;
                output.write_instruction(Unit::Main, "JMPFALSEP", &[id]);
            }
            LDGLOBAL | LDGLOBALW => {
                let (name, id) = read_wide(instr, ("LDGLOBAL", LDGLOBAL), ("LDGLOBALW", LDGLOBALW), &mut reader)?;
                let constant = StackValue::from(cp[id as usize].clone());

                let args: &[&dyn Display] = &[&StackId(stack), &constant];
                output.write_instruction(Unit::Main, name, args);
                stack += 1;
            }
            STORELOCAL | STORELOCALW => {
                let (name, id) = read_wide(
                    instr,
                    ("STORELOCAL", STORELOCAL),
                    ("STORELOCALW", STORELOCALW),
                    &mut reader,
                )?;

                let args: &[&dyn Display] = &[&StackId(id.into())];
                output.write_instruction(Unit::Main, name, args);
                stack += 1;
            }
            CALL => {
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
            STATICPROPACCESS | STATICPROPACCESSW => {
                let (name, id) = read_wide(
                    instr,
                    ("STATICPROPACCESS", STATICPROPACCESS),
                    ("STATICPROPACCESSW", STATICPROPACCESSW),
                    &mut reader,
                )?;
                let constant = StackValue::from(cp[id as usize].clone());

                let args: &[&dyn Display] = &[&StackId(stack), &constant];
                output.write_instruction(Unit::Main, name, args);
                stack += 1;
            }
            RET => {
                output.write_instruction(Unit::Main, "RET", &[StackId(stack)]);
                break;
            }
            _ => return Err(DecompileError::UnknownInstruction(instr)),
        }
    }

    Ok(output.finish())
}

#[cfg(test)]
#[test]
fn test_decompile() {
    use super::FunctionCompiler;
    use crate::optimizer;
    use crate::optimizer::consteval::OptLevel;
    use crate::parser::parser::Parser;

    let parser = Parser::from_str(
        r#"
        function fib(n) {
            if (n < 2) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(20);

    "#,
    )
    .unwrap();
    let mut ast = parser.parse_all().unwrap();
    optimizer::optimize_ast(&mut ast, OptLevel::Aggressive);
    let cmp = FunctionCompiler::new().compile_ast(ast).unwrap();
    let dec = decompile(cmp).unwrap();
    println!("{}", dec);
}
