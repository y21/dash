use std::cell::RefCell;

use dash_middle::compiler::StaticImportKind;
use dash_rt::module::ModuleLoader;
use dash_vm::localscope::LocalScope;
use dash_vm::value::string::JsString;
use dash_vm::value::{Root, Value};
use dash_vm::{Vm, throw};
use indexmap::IndexSet;

#[derive(Debug, Default)]
pub struct ScriptModule {
    import_stack: RefCell<IndexSet<String>>,
}

impl ScriptModule {
    pub fn add_import(&self, sc: &mut LocalScope, name: String) -> Result<String, Value> {
        let mut stack = self.import_stack.borrow_mut();
        if stack.contains(&name) {
            throw!(sc, Error, "import cycle detected: {}", name);
        }

        stack.insert(name.to_string());
        Ok(name)
    }

    pub fn pop_import(&self) {
        self.import_stack.borrow_mut().pop();
    }
}

impl ModuleLoader for ScriptModule {
    fn import(&self, sc: &mut LocalScope, import_ty: StaticImportKind, path: JsString) -> Result<Option<Value>, Value> {
        let path = self.add_import(sc, path.res(sc).to_owned())?;

        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => throw!(sc, ReferenceError, "{}", err),
        };
        let module = Vm::evaluate_module(sc, &contents, import_ty, Default::default()).root(sc);

        self.pop_import();

        module.map(Some)
    }
}
