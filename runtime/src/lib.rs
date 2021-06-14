use std::{cell::RefCell, rc::Rc};

use dash::{
    agent::{Agent, ImportResult},
    compiler::compiler::Compiler,
    js_std::{self, error::MaybeRc},
    parser::{lexer::Lexer, parser::Parser},
    util::MaybeOwned,
    vm::value::{
        function::{CallContext, NativeFunction},
        object::AnyObject,
        Value,
    },
};

pub mod fs;

pub mod agent_flags {
    pub const FS: u32 = 1 << 0;
    pub const FS_CACHE: u32 = 1 << 1;
}
pub struct RuntimeAgent(u32);

impl RuntimeAgent {
    fn has_flag(&self, flag: u32) -> bool {
        (self.0 & flag) == flag
    }
    fn allow_fs(&self) -> bool {
        self.has_flag(agent_flags::FS)
    }
}

fn read_file(call: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let mut args = call.arguments();
    let filename_cell = args.next();
    let filename_ref = filename_cell.map(|c| c.borrow());
    let filename = filename_ref
        .as_deref()
        .and_then(Value::as_string)
        .ok_or_else(|| {
            js_std::error::create_error(MaybeRc::Owned("path must be a string"), call.vm)
        })?;

    let content = std::fs::read_to_string(filename)
        .map_err(|e| js_std::error::create_error(MaybeRc::Owned(&e.to_string()), call.vm))?;

    Ok(Value::from(content).into())
}

impl Agent for RuntimeAgent {
    fn random(&mut self) -> Option<f64> {
        None
    }
    fn import(&mut self, module_name: &[u8]) -> Option<ImportResult> {
        match module_name {
            b"fs" if self.allow_fs() => {
                let mut obj = Value::from(AnyObject {});

                let read_file = Value::from(NativeFunction::new(
                    "readFile",
                    read_file,
                    None,
                    dash::vm::value::function::Constructor::NoCtor,
                ))
                .into();
                obj.set_property("readFile", read_file);

                Some(ImportResult::Value(obj))
            }
            [b'.', ..] => {
                let module = std::str::from_utf8(module_name).ok()?;
                let source = std::fs::read_to_string(module).ok()?;

                let tok = Lexer::new(&source).scan_all().ok()?;
                let ast = Parser::new(&source, tok).parse_all().ok()?;
                let comp = Compiler::new(ast, Some(MaybeOwned::Borrowed(self as _)), true)
                    .compile()
                    .ok()?;

                Some(ImportResult::Bytecode(comp))
            }
            _ => None,
        }
    }
}

pub fn agent(flags: u32) -> RuntimeAgent {
    RuntimeAgent(flags)
}
