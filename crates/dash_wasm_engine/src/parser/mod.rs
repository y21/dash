use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use strum_macros::FromRepr;

use self::error::Error;

pub mod error;

const MAGIC_NUMBER: u32 = 0x6d736100;
const CURRENT_WASM_VERSION: u32 = 0x1;
const TYPE_SECTION_ID: u8 = 0x1;
const IMPORT_SECTION_ID: u8 = 0x2;
const FUNCTION_SECTION_ID: u8 = 0x3;
const TABLE_SECTION_ID: u8 = 0x4;
const MEMORY_SECTION_ID: u8 = 0x5;
const GLOBAL_SECTION_ID: u8 = 0x6;
const EXPORT_SECTION_ID: u8 = 0x7;
const START_SECTION_ID: u8 = 0x8;
const ELEMENT_SECTION_ID: u8 = 0x9;
const CODE_SECTION_ID: u8 = 0xa;
const DATA_SECTION_ID: u8 = 0xb;
const FUNCTION_END_MARKER: u8 = 0xb;

#[derive(Debug, FromRepr)]
#[repr(u8)]
enum TypeKind {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
    AnyFunc = 0x70,
    Func = 0x60,
}

#[derive(Debug)]
pub struct Type {
    form: TypeKind,
    param_types: Vec<TypeKind>,
    return_type: Option<TypeKind>,
}

#[derive(Debug, FromRepr)]
#[repr(u8)]
pub enum ExternalKind {
    Function = 0,
    Table = 1,
    Memory = 2,
    Global = 3,
}

#[derive(Debug)]
pub struct Export {
    field: String,
    kind: ExternalKind,
    index: u64,
}

#[derive(Debug)]
pub struct Code {
    locals: Vec<(u64, TypeKind)>,
    body: Vec<u8>,
}

#[derive(Debug)]
pub struct Limit {
    initial: u64,
    maximum: Option<u64>,
}

#[derive(Debug)]
pub struct Memory {
    limits: Limit,
}

#[derive(Debug)]
pub struct DataSection {
    index: u64,
    offset: u64,
    data: Vec<u8>,
}

pub struct Parser<'buf> {
    input: Cursor<&'buf [u8]>,
    type_sections: Vec<Type>,
    function_sections: Vec<u64>,
    memory_sections: Vec<Memory>,
    export_sections: Vec<Export>,
    code_sections: Vec<Code>,
    data_sections: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct Program {
    types: Vec<Type>,
    functions: Vec<u64>,
    memories: Vec<Memory>,
    exports: Vec<Export>,
    code: Vec<Code>,
    data: Vec<Vec<u8>>,
}

impl<'buf> Parser<'buf> {
    pub fn new(input: &'buf [u8]) -> Self {
        Self {
            input: Cursor::new(input),
            type_sections: Vec::new(),
            function_sections: Vec::new(),
            memory_sections: Vec::new(),
            export_sections: Vec::new(),
            code_sections: Vec::new(),
            data_sections: Vec::new(),
        }
    }

    fn parse_type_kind(&mut self) -> Result<TypeKind, Error> {
        let form = self.input.read_u8()?;
        let form = TypeKind::from_repr(form).ok_or(Error::InvalidTypeKind(form))?;
        Ok(form)
    }

    fn parse_type_section(&mut self) -> Result<(), Error> {
        let type_count = leb128::read::unsigned(&mut self.input)?;

        for _ in 0..type_count {
            let form = self.parse_type_kind()?;

            let param_count = leb128::read::unsigned(&mut self.input)?;
            let mut param_types = Vec::with_capacity(param_count as usize);
            for _ in 0..param_count {
                let param_type = self.parse_type_kind()?;
                param_types.push(param_type);
            }

            let return_count = self.input.read_u8()?;
            let return_type = match return_count {
                1.. => Some(self.parse_type_kind()?),
                0 => None,
            };
            self.type_sections.push(Type {
                form,
                param_types,
                return_type,
            });
        }

        Ok(())
    }

    fn parse_function_section(&mut self) -> Result<(), Error> {
        let count = leb128::read::unsigned(&mut self.input)?;
        for _ in 0..count {
            let type_index = leb128::read::unsigned(&mut self.input)?;
            self.function_sections.push(type_index);
        }
        Ok(())
    }

    fn parse_export_section(&mut self) -> Result<(), Error> {
        let count = leb128::read::unsigned(&mut self.input)?;

        for _ in 0..count {
            let field_len = leb128::read::unsigned(&mut self.input)?;
            let field = {
                let mut field_bytes = vec![0; field_len as usize];
                self.input.read_exact(&mut field_bytes)?;
                String::from_utf8(field_bytes)?
            };

            let kind = self.input.read_u8()?;
            let kind = ExternalKind::from_repr(kind).ok_or(Error::InvalidExternalKind(kind))?;
            let index = leb128::read::unsigned(&mut self.input)?;

            self.export_sections.push(Export { field, kind, index });
        }

        Ok(())
    }

    fn parse_code_section(&mut self) -> Result<(), Error> {
        let count = leb128::read::unsigned(&mut self.input)?;

        for _ in 0..count {
            let body_size = leb128::read::unsigned(&mut self.input)?;
            let pos_before_locals = self.input.position();
            let local_count = leb128::read::unsigned(&mut self.input)?;
            let mut locals = Vec::new();
            for _ in 0..local_count {
                let count = leb128::read::unsigned(&mut self.input)?;
                let kind = self.parse_type_kind()?;
                locals.push((count, kind));
            }
            let pos_after_locals = self.input.position();
            let bytecode_size = body_size - (pos_after_locals - pos_before_locals) - 1;
            let mut bytecode = vec![0; bytecode_size as usize];
            self.input.read_exact(&mut bytecode)?;
            // End byte: 0xB
            let end_byte = self.input.read_u8()?;
            if end_byte != FUNCTION_END_MARKER {
                return Err(Error::InvalidFunctionEndMarker(end_byte));
            }

            self.code_sections.push(Code { locals, body: bytecode });
        }

        Ok(())
    }

    fn parse_memory_section(&mut self) -> Result<(), Error> {
        let count = leb128::read::unsigned(&mut self.input)?;

        for _ in 0..count {
            let flags = self.input.read_u8()?;
            let initial = leb128::read::unsigned(&mut self.input)?;
            let maximum = match flags {
                0x00 => None,
                0x01 => Some(leb128::read::unsigned(&mut self.input)?),
                _ => return Err(Error::InvalidMemoryFlags(flags)),
            };
            self.memory_sections.push(Memory {
                limits: Limit { initial, maximum },
            });
        }

        Ok(())
    }

    pub fn parse(mut self) -> Result<Program, Error> {
        let header = self.input.read_u32::<LittleEndian>()?;
        if header != MAGIC_NUMBER {
            return Err(Error::IncorrectMagicNumber(header));
        }

        let wasm_version = self.input.read_u32::<LittleEndian>()?;
        if wasm_version != CURRENT_WASM_VERSION {
            return Err(Error::UnsupportedWasmVersion(wasm_version));
        }

        loop {
            let section_id = match self.input.read_u8() {
                Ok(id) => id,
                Err(..) => break,
            };
            let payload_len = leb128::read::unsigned(&mut self.input)?;

            match section_id {
                TYPE_SECTION_ID => {
                    self.parse_type_section()?;
                }
                FUNCTION_SECTION_ID => {
                    self.parse_function_section()?;
                }
                MEMORY_SECTION_ID => {
                    self.parse_memory_section()?;
                }
                EXPORT_SECTION_ID => {
                    self.parse_export_section()?;
                }
                CODE_SECTION_ID => {
                    self.parse_code_section()?;
                }
                DATA_SECTION_ID => {
                    // Data sections require evaluating expressions, which we can't do (nicely) in the parser
                    // so we only scan the raw bytes and let the VM do this
                    let mut raw = vec![0; payload_len as usize];
                    self.input.read_exact(&mut raw)?;
                    self.data_sections.push(raw);
                }
                _ => {
                    self.input.seek(SeekFrom::Current(payload_len as i64))?;
                }
            }
        }

        Ok(Program {
            types: self.type_sections,
            functions: self.function_sections,
            exports: self.export_sections,
            code: self.code_sections,
            memories: self.memory_sections,
            data: self.data_sections,
        })
    }
}
