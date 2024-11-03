use dash_middle::interner::sym;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::{register_native_fn, CallContext};
use dash_vm::value::object::{NamedObject, Object, PropertyDataDescriptor, PropertyValue, PropertyValueKind};
use dash_vm::value::{Root, Typeof, Value};

use crate::state::{state_mut, NodeSymbols};

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols {
        inherits: inherits_sym, ..
    } = state_mut(sc).sym;
    let exports = sc.register(NamedObject::new(sc));

    let inherits = register_native_fn(sc, inherits_sym, inherits);
    exports.set_property(sc, inherits_sym.into(), PropertyValue::static_default(inherits.into()))?;

    Ok(exports.into())
}

fn inherits(cx: CallContext) -> Result<Value, Value> {
    let [ctor, super_ctor] = *cx.args else {
        throw!(cx.scope, Error, "expected 2 arguments to util.inherits")
    };

    if ctor.type_of(cx.scope) != Typeof::Function {
        throw!(cx.scope, TypeError, "expected function for the \"ctor\" argument");
    }

    if super_ctor.type_of(cx.scope) != Typeof::Function {
        throw!(cx.scope, TypeError, "expected function for the \"super_ctor\" argument");
    }

    let super_inst = super_ctor
        .construct(cx.scope, Value::undefined(), Vec::new())
        .root(cx.scope)?;

    super_inst.set_property(
        cx.scope,
        sym::constructor.into(),
        PropertyValue {
            kind: PropertyValueKind::Static(ctor),
            descriptor: PropertyDataDescriptor::WRITABLE | PropertyDataDescriptor::CONFIGURABLE,
        },
    )?;

    ctor.set_property(
        cx.scope,
        sym::prototype.into(),
        PropertyValue::static_default(super_inst),
    )?;

    Ok(Value::undefined())
}
