use dash_middle::compiler::StaticImportKind;
use dash_rt::module::ModuleLoader;
use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::string::JsString;

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
            name.to_key(sc),
            PropertyValue::static_default(Value::object(tcplistener)),
            sc,
        )?;

        Ok(Some(Value::object(sc.register(exports))))
    }
}
