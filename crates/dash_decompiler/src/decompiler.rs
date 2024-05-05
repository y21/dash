use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::instruction::{Instruction, IntrinsicOperation};
use dash_middle::compiler::{FunctionCallMetadata, ObjectMemberKind};
use dash_middle::interner::StringInterner;
use dash_middle::util::Reader;
use std::fmt;
use std::fmt::Write;
use std::rc::Rc;

use crate::DecompileError;

pub struct FunctionDecompiler<'interner, 'buf> {
    interner: &'interner StringInterner,
    reader: Reader<&'buf [u8]>,
    constants: &'buf [Constant],
    name: &'buf str,
    out: String,
    /// Index of the current instruction in the bytecode
    instr_idx: usize,
}

impl<'interner, 'buf> FunctionDecompiler<'interner, 'buf> {
    pub fn new(
        interner: &'interner StringInterner,
        buf: &'buf [u8],
        constants: &'buf [Constant],
        name: &'buf str,
    ) -> Self {
        Self {
            reader: Reader::new(buf),
            constants,
            interner,
            out: format!("function {name}:\n"),
            name,
            instr_idx: 0,
        }
    }

    fn handle_opless_instr(&mut self, name: &str) {
        let _ = writeln!(self.out, "{:02x}  {}", self.instr_idx, name);
    }

    fn handle_op_instr(&mut self, name: &str, args: &[&dyn fmt::Display]) {
        let _ = write!(self.out, "{:02x}  {}  ", self.instr_idx, name);
        for (index, arg) in args.iter().enumerate() {
            if index > 0 {
                let _ = write!(self.out, ", ");
            }

            let _ = write!(self.out, "{arg}");
        }
        let _ = self.out.write_char('\n');
    }

    fn handle_op_map_instr(&mut self, name: &str, args: &[(&str, &dyn fmt::Display)]) {
        let _ = write!(self.out, "{:02x}  {}  ", self.instr_idx, name);
        for (index, (key, arg)) in args.iter().enumerate() {
            if index > 0 {
                let _ = write!(self.out, ", ");
            }

            let _ = write!(self.out, "{key}: {arg}");
        }
        let _ = self.out.write_char('\n');
    }

    /// Handles an opcode with a single argument that is in the following bytecode.
    fn handle_inc_op_instr(&mut self, name: &str) -> Result<(), DecompileError> {
        let b = self.read()?;
        self.handle_op_instr(name, &[&b]);
        Ok(())
    }

    /// Handles an opcode with a single argument that is in the following bytecode.
    fn handle_inc_op_instr2(&mut self, name: &str) -> Result<(), DecompileError> {
        let b = self.read()?;
        let b2 = self.read()?;
        self.handle_op_instr(name, &[&b, &b2]);
        Ok(())
    }

    /// Handles an opcode with a single wide argument that is in the following bytecode.
    fn handle_incw_op_instr(&mut self, name: &str) -> Result<(), DecompileError> {
        let b = self.read_u16()?;
        self.handle_op_instr(name, &[&b]);
        Ok(())
    }

    /// Handles an opcode with a single wide argument that is in the following bytecode.
    fn handle_incw_op_instr2(&mut self, name: &str) -> Result<(), DecompileError> {
        let b = self.read_u16()?;
        let b2 = self.read()?;
        self.handle_op_instr(name, &[&b, &b2]);
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

    fn read_u32(&mut self) -> Result<u32, DecompileError> {
        self.reader.read_u32_ne().ok_or(DecompileError::AbruptEof)
    }

    fn display(&self, constant: &'buf Constant) -> DisplayConstant<'interner, 'buf> {
        DisplayConstant(self.interner, constant)
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
                Instruction::Constant => {
                    let b = self.read()?;
                    let constant = &self.constants[b as usize];
                    if let Constant::Function(fun) = constant {
                        functions.push(Rc::clone(fun));
                    }
                    self.handle_op_instr("constant", &[&self.display(constant)]);
                }
                Instruction::ConstantW => {
                    let b = self.read_u16()?;
                    let constant = &self.constants[b as usize];
                    if let Constant::Function(fun) = constant {
                        functions.push(Rc::clone(fun));
                    }
                    self.handle_op_instr("constant", &[&self.display(constant)]);
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
                | Instruction::JmpTrueP
                | Instruction::JmpUndefinedNP
                | Instruction::JmpUndefinedP => {
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
                            Instruction::JmpUndefinedP => "jmpundefinedp",
                            Instruction::JmpUndefinedNP => "jmpundefinednp",
                            _ => unreachable!(),
                        },
                        &[&arg],
                    );
                }
                Instruction::Arguments => self.handle_opless_instr("arguments"),
                Instruction::LdGlobal => {
                    let b = self.read()?;
                    self.handle_op_instr("ldglobal", &[&self.display(&self.constants[b as usize])]);
                }
                Instruction::LdGlobalW => {
                    let b = self.read_u16()?;
                    self.handle_op_instr("ldglobalw", &[&self.display(&self.constants[b as usize])]);
                }
                Instruction::StoreLocal => self.handle_inc_op_instr2("storelocal")?,
                Instruction::StoreLocalW => self.handle_inc_op_instr2("storelocalw")?,
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
                    self.handle_op_instr("staticpropaccess", &[&self.display(&self.constants[b as usize])]);
                }
                Instruction::StaticPropAccessW => {
                    let b = self.read_u16()?;
                    let _preserve_this = self.read()?;
                    self.handle_op_instr("staticpropaccessw", &[&self.display(&self.constants[b as usize])]);
                }
                Instruction::Ret => {
                    self.read_u16()?; // intentionally ignored
                    self.handle_opless_instr("ret")
                }
                Instruction::Pos => self.handle_opless_instr("pos"),
                Instruction::Neg => self.handle_opless_instr("neg"),
                Instruction::TypeOfGlobalIdent => {
                    let id = self.read_u16()?;
                    self.handle_op_instr("typeof", &[&self.display(&self.constants[id as usize])]);
                }
                Instruction::TypeOf => self.handle_opless_instr("typeof"),
                Instruction::BitNot => self.handle_opless_instr("bitnot"),
                Instruction::Not => self.handle_opless_instr("not"),
                Instruction::StoreGlobal => {
                    let b = self.read()?;
                    let _kind = self.read();
                    self.handle_op_instr("storeglobal", &[&self.display(&self.constants[b as usize])]);
                }
                Instruction::StoreGlobalW => {
                    let b = self.read_u16()?;
                    let _kind = self.read();
                    self.handle_op_instr("storeglobalw", &[&self.display(&self.constants[b as usize])]);
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
                                let cid = self.read_u16()?;
                                props.push(self.display(&self.constants[cid as usize]).to_string());
                            }
                            ObjectMemberKind::Spread => {
                                props.push(String::from("<spread>"));
                            }
                        }
                    }
                    let props = props.iter().map(|v| v as &dyn fmt::Display).collect::<Vec<_>>();
                    self.handle_op_instr("objlit", &props);
                }
                Instruction::This => self.handle_opless_instr("this"),
                Instruction::StaticPropAssign => {
                    let _k = self.read()?;
                    let b = self.read_u16()?;
                    self.handle_op_instr("staticpropassign", &[&self.display(&self.constants[b as usize])]);
                }
                Instruction::DynamicPropAssign => {
                    let _k = self.read()?;
                    self.handle_opless_instr("dynamicpropset")
                }
                Instruction::LdLocalExt => self.handle_inc_op_instr("ldlocalext")?,
                Instruction::LdLocalExtW => self.handle_incw_op_instr("ldlocalextw")?,
                Instruction::StoreLocalExt => self.handle_inc_op_instr2("storelocalext")?,
                Instruction::StoreLocalExtW => self.handle_incw_op_instr2("storelocalextw")?,
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
                Instruction::ImportStatic => {
                    let _kind = self.read()?;
                    let _local_id = self.read_i16()?;
                    let _path_id = self.read_i16()?;
                    self.handle_opless_instr("importstatic")
                }
                Instruction::ExportDefault => todo!(),
                Instruction::ExportNamed => todo!(),
                Instruction::Debugger => self.handle_opless_instr("debugger"),
                Instruction::Global => self.handle_opless_instr("global"),
                Instruction::Super => self.handle_opless_instr("super"),
                Instruction::Undef => self.handle_opless_instr("undef"),
                Instruction::Await => self.handle_opless_instr("await"),
                Instruction::Nan => self.handle_opless_instr("nan"),
                Instruction::Infinity => self.handle_opless_instr("inf"),
                Instruction::IntrinsicOp => {
                    let op =
                        IntrinsicOperation::from_repr(self.read()?).ok_or(DecompileError::InvalidObjectMemberKind)?;

                    match op {
                        IntrinsicOperation::AddNumLR => self.handle_opless_instr("iadd"),
                        IntrinsicOperation::SubNumLR => self.handle_opless_instr("isub"),
                        IntrinsicOperation::MulNumLR => self.handle_opless_instr("imul"),
                        IntrinsicOperation::DivNumLR => self.handle_opless_instr("idiv"),
                        IntrinsicOperation::RemNumLR => self.handle_opless_instr("irem"),
                        IntrinsicOperation::PowNumLR => self.handle_opless_instr("ipow"),
                        IntrinsicOperation::GtNumLR => self.handle_opless_instr("igt"),
                        IntrinsicOperation::GeNumLR => self.handle_opless_instr("ige"),
                        IntrinsicOperation::LtNumLR => self.handle_opless_instr("ilt"),
                        IntrinsicOperation::LeNumLR => self.handle_opless_instr("ile"),
                        IntrinsicOperation::EqNumLR => self.handle_opless_instr("ieq"),
                        IntrinsicOperation::NeNumLR => self.handle_opless_instr("ine"),
                        IntrinsicOperation::BitOrNumLR => self.handle_opless_instr("ibitor"),
                        IntrinsicOperation::BitXorNumLR => self.handle_opless_instr("ibitxor"),
                        IntrinsicOperation::BitAndNumLR => self.handle_opless_instr("ibitand"),
                        IntrinsicOperation::BitShlNumLR => self.handle_opless_instr("ibitshl"),
                        IntrinsicOperation::BitShrNumLR => self.handle_opless_instr("ibitshr"),
                        IntrinsicOperation::BitUshrNumLR => self.handle_opless_instr("ibitushr"),
                        IntrinsicOperation::PostfixIncLocalNum => self.handle_inc_op_instr("ipostinclocal")?,
                        IntrinsicOperation::PostfixDecLocalNum => self.handle_inc_op_instr("ipostdeclocal")?,
                        IntrinsicOperation::PrefixIncLocalNum => self.handle_inc_op_instr("ipreinclocal")?,
                        IntrinsicOperation::PrefixDecLocalNum => self.handle_inc_op_instr("ipredeclocal")?,
                        IntrinsicOperation::GtNumLConstR => self.handle_inc_op_instr("igtconst")?,
                        IntrinsicOperation::GeNumLConstR => self.handle_inc_op_instr("igeconst")?,
                        IntrinsicOperation::LtNumLConstR => self.handle_inc_op_instr("iltconst")?,
                        IntrinsicOperation::LeNumLConstR => self.handle_inc_op_instr("ileconst")?,
                        IntrinsicOperation::GtNumLConstR32 => {
                            let b = self.read_u32()?;
                            self.handle_op_instr("igtconst32", &[&b]);
                        }
                        IntrinsicOperation::GeNumLConstR32 => {
                            let b = self.read_u32()?;
                            self.handle_op_instr("igeconst32", &[&b]);
                        }
                        IntrinsicOperation::LtNumLConstR32 => {
                            let b = self.read_u32()?;
                            self.handle_op_instr("iltconst32", &[&b]);
                        }
                        IntrinsicOperation::LeNumLConstR32 => {
                            let b = self.read_u32()?;
                            self.handle_op_instr("ileconst32", &[&b]);
                        }
                        IntrinsicOperation::Exp => self.handle_inc_op_instr("exp")?,
                        IntrinsicOperation::Log2 => self.handle_inc_op_instr("log2")?,
                        IntrinsicOperation::Expm1 => self.handle_inc_op_instr("expm1")?,
                        IntrinsicOperation::Cbrt => self.handle_inc_op_instr("cbrt")?,
                        IntrinsicOperation::Clz32 => self.handle_inc_op_instr("clz32")?,
                        IntrinsicOperation::Atanh => self.handle_inc_op_instr("atanh")?,
                        IntrinsicOperation::Atan2 => self.handle_inc_op_instr("atan2")?,
                        IntrinsicOperation::Round => self.handle_inc_op_instr("round")?,
                        IntrinsicOperation::Acosh => self.handle_inc_op_instr("acosh")?,
                        IntrinsicOperation::Abs => self.handle_inc_op_instr("abs")?,
                        IntrinsicOperation::Sinh => self.handle_inc_op_instr("sinh")?,
                        IntrinsicOperation::Sin => self.handle_inc_op_instr("sin")?,
                        IntrinsicOperation::Ceil => self.handle_inc_op_instr("ceil")?,
                        IntrinsicOperation::Tan => self.handle_inc_op_instr("tan")?,
                        IntrinsicOperation::Trunc => self.handle_inc_op_instr("trunc")?,
                        IntrinsicOperation::Asinh => self.handle_inc_op_instr("asinh")?,
                        IntrinsicOperation::Log10 => self.handle_inc_op_instr("log10")?,
                        IntrinsicOperation::Asin => self.handle_inc_op_instr("asin")?,
                        IntrinsicOperation::Random => self.handle_inc_op_instr("random")?,
                        IntrinsicOperation::Log1p => self.handle_inc_op_instr("log1p")?,
                        IntrinsicOperation::Sqrt => self.handle_inc_op_instr("sqrt")?,
                        IntrinsicOperation::Atan => self.handle_inc_op_instr("atan")?,
                        IntrinsicOperation::Cos => self.handle_inc_op_instr("cos")?,
                        IntrinsicOperation::Tanh => self.handle_inc_op_instr("tanh")?,
                        IntrinsicOperation::Log => self.handle_inc_op_instr("log")?,
                        IntrinsicOperation::Floor => self.handle_inc_op_instr("floor")?,
                        IntrinsicOperation::Cosh => self.handle_inc_op_instr("cosh")?,
                        IntrinsicOperation::Acos => self.handle_inc_op_instr("acos")?,
                    }
                }
                Instruction::CallSymbolIterator => self.handle_opless_instr("@@iterator"),
                Instruction::CallForInIterator => self.handle_opless_instr("@@forInIterator"),
                Instruction::DeletePropertyStatic => self.handle_incw_op_instr("deletepropertystatic")?,
                Instruction::DeletePropertyDynamic => self.handle_opless_instr("deletepropertydynamic"),
                Instruction::ObjDestruct => {
                    let count = self.read_u16()?;
                    for _ in 0..count {
                        self.read_u16()?; // discard var id
                        self.read_u16()?; // discard property name id
                    }
                    self.handle_op_map_instr("objdestruct", &[("count", &count)])
                }
                Instruction::ArrayDestruct => {
                    let count = self.read_u16()?;
                    for _ in 0..count {
                        self.read_u16()?; // discard var id
                    }
                    self.handle_op_map_instr("arraydestruct", &[("count", &count)])
                }
                Instruction::AssignProperties => todo!(),
                Instruction::Nop => self.handle_opless_instr("nop"),
            }
        }

        // Finally, append all other functions defined in this function

        for fun in functions {
            let out = fun.buffer.with(|buffer| {
                FunctionDecompiler::new(
                    self.interner,
                    buffer,
                    &fun.constants,
                    &format!("{}::{:?}", self.name, fun.name),
                )
                .run()
            })?;
            self.out.push('\n');
            self.out.push_str(&out);
        }

        Ok(self.out)
    }
}

struct DisplayConstant<'i, 'a>(&'i StringInterner, &'a Constant);
impl fmt::Display for DisplayConstant<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            Constant::Number(n) => write!(f, "{n}"),
            Constant::String(s) => write!(f, "\"{}\"", self.0.resolve(*s)),
            Constant::Boolean(b) => write!(f, "{b}"),
            Constant::Identifier(ident) => write!(f, "{}", self.0.resolve(*ident)),
            Constant::Function(fun) => write!(
                f,
                "<function {}>",
                fun.name.map(|v| self.0.resolve(v)).unwrap_or("<anon>")
            ),
            Constant::Null => f.write_str("null"),
            Constant::Undefined => f.write_str("undefined"),
            Constant::Regex(regex) => {
                let (_, _, sym) = &**regex;
                write!(f, "{}", self.0.resolve(*sym))
            }
        }
    }
}
