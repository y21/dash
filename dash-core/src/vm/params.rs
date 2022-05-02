use crate::compiler::StaticImportKind;

use super::value::Value;
use super::Vm;

pub type StaticImportCallback = fn(vm: &mut Vm, ty: StaticImportKind, path: &str) -> Result<Value, Value>;
pub type DynamicImportCallback = fn(vm: &mut Vm, val: Value) -> Result<Value, Value>;

#[derive(Default)]
pub struct VmParams {
    static_import_callback: Option<StaticImportCallback>,
    dynamic_import_callback: Option<DynamicImportCallback>,
}

impl VmParams {
    pub fn new() -> Self {
        VmParams::default()
    }

    pub fn set_static_import_callback(mut self, callback: StaticImportCallback) -> Self {
        self.static_import_callback = Some(callback);
        self
    }

    pub fn set_dynamic_import_callback(mut self, callback: DynamicImportCallback) -> Self {
        self.dynamic_import_callback = Some(callback);
        self
    }

    pub fn static_import_callback(&self) -> Option<StaticImportCallback> {
        self.static_import_callback
    }

    pub fn dynamic_import_callback(&self) -> Option<DynamicImportCallback> {
        self.dynamic_import_callback
    }
}
