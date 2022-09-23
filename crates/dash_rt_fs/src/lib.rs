use dash_middle::compiler::StaticImportKind;
use dash_rt::module::ModuleLoader;
use dash_vm::local::LocalScope;
use dash_vm::value::Value;

pub mod promises;
pub mod sync;

#[derive(Debug)]
pub struct FsModule;

impl ModuleLoader for FsModule {
    fn import(&self, sc: &mut LocalScope, _import_ty: StaticImportKind, path: &str) -> Option<Value> {
        match path {
            "@std/fs" => promises::init_module(sc),
            "@std/fssync" => sync::init_module(sc),
            _ => None,
        }
    }
}
