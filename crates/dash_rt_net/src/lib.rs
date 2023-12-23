use dash_middle::compiler::StaticImportKind;
use dash_rt::module::ModuleLoader;
use dash_vm::localscope::LocalScope;
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::Value;

use crate::listener::TcpListenerConstructor;

mod listener;

#[derive(Debug)]
pub struct NetModule;

impl ModuleLoader for NetModule {
    fn import(&self, sc: &mut LocalScope, _import_ty: StaticImportKind, path: &str) -> Result<Option<Value>, Value> {
        if path != "@std/net" {
            return Ok(None);
        }

        let exports = NamedObject::new(sc);
        let tcplistener = sc.register(TcpListenerConstructor {});
        exports.set_property(
            sc,
            "TcpListener".into(),
            PropertyValue::static_default(Value::Object(tcplistener)),
        )?;

        Ok(Some(Value::Object(sc.register(exports))))
    }
}
