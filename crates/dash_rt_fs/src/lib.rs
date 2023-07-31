use dash_middle::compiler::StaticImportKind;
use dash_rt::module::ModuleLoader;
use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;

pub mod promises;
pub mod sync;

#[derive(Debug)]
pub struct FsModule;

impl ModuleLoader for FsModule {
    fn import(&self, sc: &mut LocalScope, _import_ty: StaticImportKind, path: &str) -> Result<Option<Value>, Value> {
        match path {
            "@std/fs" => promises::init_module(sc).map(Some),
            "@std/fssync" => sync::init_module(sc).map(Some),
            _ => Ok(None),
        }
    }
}
