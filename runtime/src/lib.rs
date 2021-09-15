use dash::{
    agent::{Agent, ImportResult},
    compiler::compiler::{Compiler, FunctionKind},
    gc::{Gc, Handle},
    js_std,
    parser::{lexer::Lexer, parser::Parser},
    util::MaybeOwned,
    vm::value::{
        function::{CallContext, NativeFunction},
        object::Object,
        Value,
    },
};

pub mod fs;

pub mod agent_flags {
    pub const FS: u32 = 1 << 0;
    pub const FS_CACHE: u32 = 1 << 1;
    pub const MEM: u32 = 1 << 2;
}
pub struct RuntimeAgent(u32);

impl RuntimeAgent {
    fn has_flag(&self, flag: u32) -> bool {
        (self.0 & flag) == flag
    }
    fn allow_fs(&self) -> bool {
        self.has_flag(agent_flags::FS)
    }
    fn allow_mem(&self) -> bool {
        self.has_flag(agent_flags::MEM)
    }
}

fn read_file(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let mut args = ctx.arguments();
    let filename_cell = args.next();
    let filename_ref = filename_cell.map(|c| unsafe { c.borrow_unbounded() });
    let filename = filename_ref
        .as_ref()
        .map(|x| &***x)
        .and_then(Value::as_string)
        .ok_or_else(|| js_std::error::create_error("path must be a string", ctx.vm))?;

    let content = std::fs::read_to_string(filename)
        .map_err(|e| js_std::error::create_error(e.to_string(), ctx.vm))?;

    Ok(ctx.vm.create_js_value(content).into_handle(ctx.vm))
}

fn mem_address_of(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let handle = ctx.args.first().unwrap();
    let ptr = handle.as_ptr() as usize as f64;
    Ok(ctx.vm.create_js_value(ptr).into_handle(ctx.vm))
}

impl Agent for RuntimeAgent {
    fn random(&mut self) -> Option<f64> {
        None
    }
    fn import(&mut self, module_name: &[u8], gc: &mut Gc<Value>) -> Option<ImportResult> {
        match module_name {
            b"fs" if self.allow_fs() => {
                let mut obj = Value::from(Object::Ordinary);

                let read_file = Value::from(NativeFunction::new(
                    "readFile",
                    read_file,
                    None,
                    dash::vm::value::function::Constructor::NoCtor,
                ));
                obj.set_property("readFile".into(), gc.register(read_file));

                Some(ImportResult::Value(obj))
            }
            b"mem" if self.allow_mem() => {
                let mut obj = Value::from(Object::Ordinary);

                let address_of = Value::from(NativeFunction::new(
                    "addressOf",
                    mem_address_of,
                    None,
                    dash::vm::value::function::Constructor::NoCtor,
                ));
                obj.set_property("addressOf".into(), gc.register(address_of));

                Some(ImportResult::Value(obj))
            }
            [b'.', ..] => {
                let module = std::str::from_utf8(module_name).ok()?;
                let source = std::fs::read_to_string(module).ok()?;

                let tok = Lexer::new(&source).scan_all().ok()?;
                let ast = Parser::new(&source, tok).parse_all().ok()?;
                let (buffer, constants, module_gc) = Compiler::new(
                    ast,
                    Some(MaybeOwned::Borrowed(self as _)),
                    FunctionKind::Module,
                )
                .compile()
                .ok()?;

                // Transfer all handles from the module GC to this GC
                // This is required, because otherwise module_gc will
                // deallocate all of its handles in its destructor, which
                // is going to be problematic when we later try to use it
                gc.transfer(module_gc);

                Some(ImportResult::Bytecode(buffer, constants.into()))
            }
            _ => None,
        }
    }
}

pub fn agent(flags: u32) -> RuntimeAgent {
    RuntimeAgent(flags)
}
