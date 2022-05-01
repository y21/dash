use super::value::Value;
use super::Vm;

pub type ImportCallback = fn(vm: &mut Vm, ty: u8, path: &str) -> Result<Value, Value>;

#[derive(Default)]
pub struct VmParams {
    import_callback: Option<ImportCallback>,
}

impl VmParams {
    pub fn new() -> Self {
        VmParams::default()
    }

    pub fn set_import_callback(mut self, callback: ImportCallback) -> Self {
        self.import_callback = Some(callback);
        self
    }

    pub fn import_callback(&self) -> Option<ImportCallback> {
        self.import_callback
    }
}
