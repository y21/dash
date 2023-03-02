use std::cell::RefCell;

use dash_middle::compiler::StaticImportKind;
use dash_rt::module::ModuleLoader;
use dash_vm::local::LocalScope;
use dash_vm::throw;
use dash_vm::value::Value;
use dash_vm::Vm;
use indexmap::IndexSet;

#[derive(Debug)]
pub struct ScriptModule {
    import_stack: RefCell<IndexSet<String>>,
}

impl ScriptModule {
    pub fn new() -> Self {
        Self {
            import_stack: RefCell::new(IndexSet::new()),
        }
    }

    pub fn add_import(&self, sc: &mut LocalScope, name: &str) -> Result<(), Value> {
        let mut stack = self.import_stack.borrow_mut();
        if stack.contains(name) {
            throw!(sc, Error, "import cycle detected: {}", name);
        }

        stack.insert(name.to_string());
        Ok(())
    }

    pub fn pop_import(&self) {
        self.import_stack.borrow_mut().pop();
    }
}

impl ModuleLoader for ScriptModule {
    fn import(&self, sc: &mut LocalScope, import_ty: StaticImportKind, path: &str) -> Result<Option<Value>, Value> {
        self.add_import(sc, path)?;

        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => throw!(sc, ReferenceError, "{}", err),
        };
        let module = Vm::evaluate_module(sc, &contents, import_ty, Default::default());

        self.pop_import();

        module.map(Some)
    }
}
