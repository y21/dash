use std::fmt::Debug;

use dash_middle::compiler::StaticImportKind;
use dash_vm::local::LocalScope;
use dash_vm::value::Value;

pub trait ModuleLoader: Debug {
    fn import(&self, sc: &mut LocalScope, import_ty: StaticImportKind, path: &str) -> Option<Value>;

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
    fn import(&self, sc: &mut LocalScope, import_ty: StaticImportKind, path: &str) -> Option<Value> {
        self.m1
            .import(sc, import_ty, path)
            .or_else(|| self.m2.import(sc, import_ty, path))
    }
}
