use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::FunctionCallMetadata;
use dash_middle::compiler::ObjectMemberKind;
use dash_middle::util::Reader;
use std::fmt;
use std::fmt::Write;
use std::rc::Rc;

use crate::DecompileError;

pub struct FunctionDecompiler<'buf> {
    reader: Reader<&'buf [u8]>,
    constants: &'buf [Constant],
    name: &'buf str,
    out: String,
    /// Index of the current instruction in the bytecode
    instr_idx: usize,
}

impl<'buf> FunctionDecompiler<'buf> {
    pub fn new(buf: &'buf [u8], constants: &'buf [Constant], name: &'buf str) -> Self {
        Self {
            reader: Reader::new(buf),
            constants,
            out: format!("function {name}:\n"),
            name,
            instr_idx: 0,
        }
    }

    fn handle_opless_instr(&mut self, name: &str) {
        let _ = write!(self.out, "{:02x}  {}\n", self.instr_idx, name);
    }

    fn handle_op_instr(&mut self, name: &str, args: &[&dyn fmt::Display]) {
        let _ = write!(self.out, "{:02x}  {}  ", self.instr_idx, name);
        for (index, arg) in args.iter().enumerate() {
            if index > 0 {
                let _ = write!(self.out, ", ");
            }

            let _ = write!(self.out, "{}", arg);
        }
        let _ = self.out.write_char('\n');
    }

    fn handle_op_map_instr(&mut self, name: &str, args: &[(&str, &dyn fmt::Display)]) {
        let _ = write!(self.out, "{:02x}  {}  ", self.instr_idx, name);
        for (index, (key, arg)) in args.iter().enumerate() {
            if index > 0 {
                let _ = write!(self.out, ", ");
            }

            let _ = write!(self.out, "{}: {}", key, arg);
        }
        let _ = self.out.write_char('\n');
    }

    /// Handles an opcode with a single argument that is in the following bytecode.
    fn handle_inc_op_instr(&mut self, name: &str) -> Result<(), DecompileError> {
        let b = self.read()?;
        self.handle_op_instr(name, &[&b]);
        Ok(())
    }

    /// Handles an opcode with a single wide argument that is in the following bytecode.
    fn handle_incw_op_instr(&mut self, name: &str) -> Result<(), DecompileError> {
        let b = self.read_u16()?;
        self.handle_op_instr(name, &[&b]);
        Ok(())
    }

    fn read(&mut self) -> Result<u8, DecompileError> {
        self.reader.read().ok_or(DecompileError::AbruptEof)
    }

    fn read_u16(&mut self) -> Result<u16, DecompileError> {
        self.reader.read_u16_ne().ok_or(DecompileError::AbruptEof)
    }

    fn read_i16(&mut self) -> Result<i16, DecompileError> {
        self.reader.read_i16_ne().ok_or(DecompileError::AbruptEof)
    }

    pub fn run(mut self) -> Result<String, DecompileError> {
        let mut functions = Vec::new();

        loop {
            self.instr_idx = self.reader.offset();
            let instr = match self.read() {
                Ok(i) => Instruction::from_repr(i).unwrap(),
                Err(..) => break,
            };

            match instr {
                Instruction::Add => self.handle_opless_instr("add"),
                Instruction::Sub => self.handle_opless_instr("sub"),
                Instruction::Mul => self.handle_opless_instr("mul"),
                Instruction::Div => self.handle_opless_instr("div"),
                Instruction::Rem => self.handle_opless_instr("rem"),
                Instruction::Pow => self.handle_opless_instr("pow"),
                Instruction::Gt => self.handle_opless_instr("gt"),
                Instruction::Ge => self.handle_opless_instr("ge"),
                Instruction::Lt => self.handle_opless_instr("lt"),
                Instruction::Le => self.handle_opless_instr("le"),
                Instruction::Eq => self.handle_opless_instr("eq"),
                Instruction::Ne => self.handle_opless_instr("ne"),
                Instruction::Pop => self.handle_opless_instr("pop"),
                Instruction::RevStck => self.handle_inc_op_instr("revstck")?,
                Instruction::Constant => {
                    let b = self.read()?;
                    let constant = &self.constants[b as usize];
                    if let Constant::Function(fun) = constant {
                        functions.push(Rc::clone(fun));
                    }
                    self.handle_op_instr("constant", &[&DisplayConstant(constant)]);
                }
                Instruction::ConstantW => {
                    let b = self.read_u16()?;
                    let constant = &self.constants[b as usize];
                    if let Constant::Function(fun) = constant {
                        functions.push(Rc::clone(fun));
                    }
                    self.handle_op_instr("constant", &[&DisplayConstant(constant)]);
                }
                Instruction::LdLocal => {
                    let b = self.read()?;
                    // TODO: use debug symbols to find the name
                    self.handle_op_instr("ldlocal", &[&b]);
                }
                Instruction::LdLocalW => self.handle_incw_op_instr("ldlocalw")?,
                Instruction::Jmp
                | Instruction::JmpFalseNP
                | Instruction::JmpFalseP
                | Instruction::JmpNullishNP
                | Instruction::JmpNullishP
                | Instruction::JmpTrueNP
                | Instruction::JmpTrueP => {
                    let byte = self.read_i16()?;
                    let offset = (self.reader.offset() as isize) + byte as isize;
                    let arg = format!("@{offset:x}");
                    self.handle_op_instr(
                        match instr {
                            Instruction::Jmp => "jmp",
                            Instruction::JmpFalseNP => "jmpfalsenp",
                            Instruction::JmpFalseP => "jmpfalsep",
                            Instruction::JmpNullishNP => "jmpnullishnp",
                            Instruction::JmpNullishP => "jmpnullishp",
                            Instruction::JmpTrueNP => "jmtruenp",
                            Instruction::JmpTrueP => "jmtruep",
                            _ => unreachable!(),
                        },
                        &[&arg],
                    );
                }
                Instruction::LdGlobal => {
                    let b = self.read()?;
                    self.handle_op_instr("ldglobal", &[&DisplayConstant(&self.constants[b as usize])]);
                }
                Instruction::LdGlobalW => {
                    let b = self.read_u16()?;
                    self.handle_op_instr("ldglobalw", &[&DisplayConstant(&self.constants[b as usize])]);
                }
                Instruction::StoreLocal => self.handle_inc_op_instr("storelocal")?,
                Instruction::StoreLocalW => self.handle_inc_op_instr("storelocalw")?,
                Instruction::Call => {
                    let meta = FunctionCallMetadata::from(self.read()?);
                    self.handle_op_map_instr(
                        "call",
                        &[
                            ("argc", &meta.value()),
                            ("is_constructor_call", &meta.is_constructor_call()),
                        ],
                    );
                }
                Instruction::StaticPropAccess => {
                    let b = self.read()?;
                    let _preserve_this = self.read()?;
                    self.handle_op_instr("staticpropaccess", &[&DisplayConstant(&self.constants[b as usize])]);
                }
                Instruction::StaticPropAccessW => {
                    let b = self.read_u16()?;
                    let _preserve_this = self.read()?;
                    self.handle_op_instr("staticpropaccessw", &[&DisplayConstant(&self.constants[b as usize])]);
                }
                Instruction::Ret => {
                    self.read_u16()?; // intentionally ignored
                    self.handle_opless_instr("ret")
                }
                Instruction::Pos => self.handle_opless_instr("pos"),
                Instruction::Neg => self.handle_opless_instr("neg"),
                Instruction::TypeOf => self.handle_opless_instr("typeof"),
                Instruction::BitNot => self.handle_opless_instr("bitnot"),
                Instruction::Not => self.handle_opless_instr("not"),
                Instruction::StoreGlobal => {
                    let b = self.read()?;
                    self.handle_op_instr("storeglobal", &[&DisplayConstant(&self.constants[b as usize])]);
                }
                Instruction::StoreGlobalW => {
                    let b = self.read_u16()?;
                    self.handle_op_instr("storeglobalw", &[&DisplayConstant(&self.constants[b as usize])]);
                }
                Instruction::DynamicPropAccess => {
                    let b = self.read()?;
                    self.handle_op_map_instr("dynamicpropaccess", &[("preserve_this", &(b == 1))])
                }
                Instruction::ArrayLit => self.handle_inc_op_instr("arraylit")?,
                Instruction::ArrayLitW => self.handle_incw_op_instr("arraylitw")?,
                Instruction::ObjLit => {
                    let len = self.read()?;
                    let mut props = Vec::new();
                    for _ in 0..len {
                        let pty =
                            ObjectMemberKind::from_repr(self.read()?).ok_or(DecompileError::InvalidObjectMemberKind)?;

                        match pty {
                            ObjectMemberKind::Dynamic => {
                                props.push(String::from("<dynamic>"));
                            }
                            ObjectMemberKind::Static | ObjectMemberKind::Getter | ObjectMemberKind::Setter => {
                                let cid = self.read()?;
                                props.push(DisplayConstant(&self.constants[cid as usize]).to_string());
                            }
                        }
                    }
                    let props = props.iter().map(|v| v as &dyn fmt::Display).collect::<Vec<_>>();
                    self.handle_op_instr("objlit", &props);
                }
                Instruction::ObjLitW => todo!(),
                Instruction::This => self.handle_opless_instr("this"),
                Instruction::StaticPropSet => {
                    let b = self.read()?;
                    self.handle_op_instr("staticpropset", &[&DisplayConstant(&self.constants[b as usize])]);
                }
                Instruction::StaticPropSetW => {
                    let b = self.read_u16()?;
                    self.handle_op_instr("staticpropsetw", &[&DisplayConstant(&self.constants[b as usize])]);
                }
                Instruction::DynamicPropSet => self.handle_opless_instr("dynamicpropset"),
                Instruction::LdLocalExt => self.handle_inc_op_instr("ldlocalext")?,
                Instruction::LdLocalExtW => self.handle_incw_op_instr("ldlocalextw")?,
                Instruction::StoreLocalExt => self.handle_inc_op_instr("storelocalext")?,
                Instruction::StoreLocalExtW => self.handle_incw_op_instr("storelocalextw")?,
                Instruction::StrictEq => self.handle_opless_instr("stricteq"),
                Instruction::StrictNe => self.handle_opless_instr("strictne"),
                Instruction::Try => self.handle_incw_op_instr("try")?, // TODO: show @offset like in JMP
                Instruction::TryEnd => self.handle_opless_instr("tryend"),
                Instruction::Throw => self.handle_opless_instr("throw"),
                Instruction::Yield => self.handle_opless_instr("yield"),
                Instruction::BitOr => self.handle_opless_instr("bitor"),
                Instruction::BitXor => self.handle_opless_instr("xor"),
                Instruction::BitAnd => self.handle_opless_instr("bitand"),
                Instruction::BitShl => self.handle_opless_instr("shl"),
                Instruction::BitShr => self.handle_opless_instr("shr"),
                Instruction::BitUshr => self.handle_opless_instr("ushr"),
                Instruction::ObjIn => self.handle_opless_instr("objin"),
                Instruction::InstanceOf => self.handle_opless_instr("instanceof"),
                Instruction::ImportDyn => todo!(),
                Instruction::ImportStatic => todo!(),
                Instruction::ExportDefault => todo!(),
                Instruction::ExportNamed => todo!(),
                Instruction::Debugger => self.handle_opless_instr("debugger"),
                Instruction::Global => self.handle_opless_instr("global"),
                Instruction::Super => self.handle_opless_instr("super"),
                Instruction::Undef => self.handle_opless_instr("undef"),
                Instruction::Break => self.handle_opless_instr("break"),
                Instruction::Await => self.handle_opless_instr("await"),
                Instruction::Nan => self.handle_opless_instr("nan"),
                Instruction::Infinity => self.handle_opless_instr("inf"),
                Instruction::IntrinsicOp => return Err(DecompileError::Unimplemented(instr)),
                Instruction::CallSymbolIterator => self.handle_opless_instr("@@iterator"),
                Instruction::CallForInIterator => self.handle_opless_instr("@@forInIterator"),
                Instruction::DeletePropertyStatic => self.handle_incw_op_instr("deletepropertystatic")?,
                Instruction::DeletePropertyDynamic => self.handle_opless_instr("deletepropertydynamic"),
                Instruction::Switch => {
                    let case_count = self.read_u16()?;
                    let has_default = self.read()? == 1;

                    for _ in 0..case_count {
                        self.read_u16()?; // discard case offsets for now..
                    }

                    if has_default {
                        self.read_u16()?;
                    }

                    self.handle_op_map_instr("switch", &[("case_count", &case_count), ("has_default", &has_default)])
                }
            }
        }

        // Finally, append all other functions defined in this function

        for fun in functions {
            let out = FunctionDecompiler::new(
                &fun.buffer,
                &fun.constants,
                &format!("{}::{}", self.name, fun.name.as_deref().unwrap_or("<anon>")),
            )
            .run()?;
            self.out.push('\n');
            self.out.push_str(&out);
        }

        Ok(self.out)
    }
}

struct DisplayConstant<'c>(&'c Constant);
impl fmt::Display for DisplayConstant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Constant::Number(n) => write!(f, "{}", n),
            Constant::String(s) => write!(f, "\"{}\"", s),
            Constant::Boolean(b) => write!(f, "{}", b),
            Constant::Identifier(ident) => write!(f, "{ident}"),
            Constant::Function(fun) => write!(f, "<function {}>", fun.name.as_deref().unwrap_or("<anonymous>")),
            Constant::Null => f.write_str("null"),
            Constant::Undefined => f.write_str("undefined"),
        }
    }
}
