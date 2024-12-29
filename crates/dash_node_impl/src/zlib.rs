use dash_proc_macro::Trace;
use dash_rt::state::State;
use dash_rt::typemap::Key;
use dash_vm::gc::ObjectId;
use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::{delegate, extract};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols {
        Inflate: inflate_sym, ..
    } = state_mut(sc).sym;

    let inflate_prototype = sc.register(Inflate {
        object: NamedObject::new(sc),
    });

    let inflate_ctor = Function::new(
        sc,
        Some(inflate_sym.into()),
        FunctionKind::Native(|cx| {
            let ZlibState {
                inflate_prototype,
                inflate_ctor,
            } = State::from_vm(cx.scope).store[ZlibKey];

            Ok(cx
                .scope
                .register(Inflate {
                    object: NamedObject::with_prototype_and_constructor(inflate_prototype, inflate_ctor),
                })
                .into())
        }),
    );
    inflate_ctor.set_fn_prototype(inflate_prototype);
    let inflate_ctor = sc.register(inflate_ctor);

    let exports = sc.register(NamedObject::new(sc));
    exports.set_property(
        inflate_sym.into(),
        PropertyValue::static_default(inflate_ctor.into()),
        sc,
    )?;

    State::from_vm_mut(sc).store.insert(ZlibKey, ZlibState {
        inflate_prototype,
        inflate_ctor,
    });

    Ok(exports.into())
}

struct ZlibKey;
impl Key for ZlibKey {
    type State = ZlibState;
}

#[derive(Debug, Trace)]
struct ZlibState {
    inflate_prototype: ObjectId,
    inflate_ctor: ObjectId,
}

#[derive(Debug, Trace)]
struct Inflate {
    object: NamedObject,
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
