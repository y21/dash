use dash_middle::compiler::StaticImportKind;
use dash_rt::module::ModuleLoader;
use dash_vm::localscope::LocalScope;
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::string::JsString;
use dash_vm::value::Value;

use crate::listener::TcpListenerConstructor;

mod listener;

#[derive(Debug)]
pub struct NetModule;

impl ModuleLoader for NetModule {
    fn import(
        &self,
        sc: &mut LocalScope,
        _import_ty: StaticImportKind,
        path: JsString,
    ) -> Result<Option<Value>, Value> {
        if path.res(sc) != "@std/net" {
            return Ok(None);
        }

        let exports = NamedObject::new(sc);
        let tcplistener = sc.register(TcpListenerConstructor {});
        let name = sc.intern("TcpListener");
        exports.set_property(
            sc,
            name.into(),
            PropertyValue::static_default(Value::object(tcplistener)),
        )?;

        Ok(Some(Value::object(sc.register(exports))))
    }
}
