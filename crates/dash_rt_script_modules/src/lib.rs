use dash_middle::compiler::StaticImportKind;
use dash_rt::module::ModuleLoader;
use dash_vm::local::LocalScope;
use dash_vm::throw;
use dash_vm::value::Value;
use dash_vm::Vm;

#[derive(Debug)]
pub struct ScriptModule;

impl ModuleLoader for ScriptModule {
    fn import(&self, sc: &mut LocalScope, import_ty: StaticImportKind, path: &str) -> Result<Option<Value>, Value> {
        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => throw!(sc, "{}", err),
        };
        let module = Vm::evaluate_module(sc, &contents, import_ty, Default::default())?;

        Ok(Some(module))
    }
}
