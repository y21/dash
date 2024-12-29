use dash_middle::interner::sym;
use dash_vm::frame::This;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::args::CallArgs;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{NamedObject, Object, PropertyDataDescriptor, PropertyValue, PropertyValueKind};
use dash_vm::value::{Root, Typeof, Value, ValueContext};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols {
        inherits: inherits_sym,
        inspect: inspect_sym,
        ..
    } = state_mut(sc).sym;
    let exports = sc.register(NamedObject::new(sc));

    let inherits = register_native_fn(sc, inherits_sym, inherits);
    let inspect = register_native_fn(sc, inspect_sym, inspect);
    exports.set_property(sc, inherits_sym.into(), PropertyValue::static_default(inherits.into()))?;
    exports.set_property(sc, inspect_sym.into(), PropertyValue::static_default(inspect.into()))?;

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
        .construct(cx.scope, This::Default, CallArgs::empty())
        .root(cx.scope)?;

    super_inst.set_property(cx.scope, sym::constructor.into(), PropertyValue {
        kind: PropertyValueKind::Static(ctor),
        descriptor: PropertyDataDescriptor::WRITABLE | PropertyDataDescriptor::CONFIGURABLE,
    })?;

    ctor.set_property(
        cx.scope,
        sym::prototype.into(),
        PropertyValue::static_default(super_inst),
    )?;

    Ok(Value::undefined())
}

fn inspect(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    let formatted = dash_rt::format_value(value, cx.scope)?.to_owned();
    Ok(Value::string(cx.scope.intern(formatted).into()))
}
