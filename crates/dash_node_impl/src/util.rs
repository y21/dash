use dash_middle::interner::sym;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::args::CallArgs;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{Object, OrdObject, PropertyDataDescriptor, PropertyValue, PropertyValueKind, This};
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::{Root, Typeof, Value, ValueContext};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols {
        inherits: inherits_sym,
        inspect: inspect_sym,
        ..
    } = state_mut(sc).sym;
    let exports = sc.register(OrdObject::new(sc));

    let inherits = register_native_fn(sc, inherits_sym, inherits);
    let inspect = register_native_fn(sc, inspect_sym, inspect);
    exports.set_property(
        inherits_sym.to_key(sc),
        PropertyValue::static_default(inherits.into()),
        sc,
    )?;
    exports.set_property(
        inspect_sym.to_key(sc),
        PropertyValue::static_default(inspect.into()),
        sc,
    )?;

    Ok(exports.into())
}

fn inherits(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let [ctor, super_ctor] = *cx.args else {
        throw!(scope, Error, "expected 2 arguments to util.inherits")
    };

    if ctor.type_of(scope) != Typeof::Function {
        throw!(scope, TypeError, "expected function for the \"ctor\" argument");
    }

    if super_ctor.type_of(scope) != Typeof::Function {
        throw!(scope, TypeError, "expected function for the \"super_ctor\" argument");
    }

    let super_inst = super_ctor
        .construct(This::default(), CallArgs::empty(), scope)
        .root(scope)?;

    super_inst.set_property(
        sym::constructor.to_key(scope),
        PropertyValue {
            kind: PropertyValueKind::Static(ctor),
            descriptor: PropertyDataDescriptor::WRITABLE | PropertyDataDescriptor::CONFIGURABLE,
        },
        scope,
    )?;

    ctor.set_property(
        sym::prototype.to_key(scope),
        PropertyValue::static_default(super_inst),
        scope,
    )?;

    Ok(Value::undefined())
}

fn inspect(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    let formatted = dash_rt::format_value(value, scope)?.to_owned();
    Ok(Value::string(scope.intern(formatted).into()))
}
