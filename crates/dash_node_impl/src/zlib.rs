use dash_proc_macro::Trace;
use dash_rt::state::State;
use dash_rt::typemap::Key;
use dash_vm::gc::ObjectId;
use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{Object, OrdObject, PropertyValue};
use dash_vm::value::propertykey::{PropertyKey, ToPropertyKey};
use dash_vm::{delegate, extract};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols {
        Inflate: inflate_sym, ..
    } = state_mut(sc).sym;

    let inflate_prototype = sc.register(Inflate {
        object: OrdObject::new(sc),
    });

    let inflate_ctor = Function::new(
        sc,
        Some(inflate_sym.into()),
        FunctionKind::Native(|_, scope| {
            let ZlibState { inflate_prototype } = State::from_vm(scope).store[ZlibKey];

            Ok(scope
                .register(Inflate {
                    object: OrdObject::with_prototype(inflate_prototype),
                })
                .into())
        }),
    );
    inflate_ctor.set_fn_prototype(inflate_prototype);
    let inflate_ctor = sc.register(inflate_ctor);
    inflate_prototype.set_property(
        PropertyKey::CONSTRUCTOR,
        PropertyValue::static_default(inflate_ctor.into()),
        sc,
    )?;

    let exports = sc.register(OrdObject::new(sc));
    exports.set_property(
        inflate_sym.to_key(sc),
        PropertyValue::static_default(inflate_ctor.into()),
        sc,
    )?;

    State::from_vm_mut(sc)
        .store
        .insert(ZlibKey, ZlibState { inflate_prototype });

    Ok(exports.into())
}

struct ZlibKey;
impl Key for ZlibKey {
    type State = ZlibState;
}

#[derive(Debug, Trace)]
struct ZlibState {
    inflate_prototype: ObjectId,
}

#[derive(Debug, Trace)]
struct Inflate {
    object: OrdObject,
}

impl Object for Inflate {
    delegate!(
        object,
        get_own_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        apply,
        own_keys
    );

    extract!(self);
}
