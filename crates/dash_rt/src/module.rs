use std::fmt::Debug;

use dash_middle::compiler::StaticImportKind;
use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;

#[derive(Debug)]
pub struct NoopModule;

impl ModuleLoader for NoopModule {
    fn import(&self, _: &mut LocalScope, _: StaticImportKind, _: &str) -> Result<Option<Value>, Value> {
        Ok(None)
    }
}

pub trait ModuleLoader: Debug {
    fn import(&self, sc: &mut LocalScope, import_ty: StaticImportKind, path: &str) -> Result<Option<Value>, Value>;

    fn or<M: ModuleLoader>(self, other: M) -> Or<Self, M>
    where
        Self: Sized,
    {
        Or { m1: self, m2: other }
    }
}

#[derive(Debug)]
pub struct Or<M1, M2> {
    m1: M1,
    m2: M2,
}

impl<M1: ModuleLoader, M2: ModuleLoader> ModuleLoader for Or<M1, M2> {
    fn import(&self, sc: &mut LocalScope, import_ty: StaticImportKind, path: &str) -> Result<Option<Value>, Value> {
        let m1 = self.m1.import(sc, import_ty, path)?;
        if m1.is_some() {
            return Ok(m1);
        }

        let m2 = self.m2.import(sc, import_ty, path)?;
        Ok(m2)
    }
}
