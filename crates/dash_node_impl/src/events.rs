use std::cell::RefCell;
use std::collections::hash_map::Entry;

use dash_middle::interner::Symbol;
use dash_proc_macro::Trace;
use dash_rt::state::State;
use dash_rt::typemap::Key;
use dash_vm::gc::handle::Handle;
use dash_vm::localscope::LocalScope;
use dash_vm::value::function::native::{register_native_fn, CallContext};
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::root_ext::RootErrExt;
use dash_vm::value::Value;
use dash_vm::{delegate, throw};
use rustc_hash::FxHashMap;

use crate::state::{state_mut, NodeSymbols};

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols {
        EventEmitter: event_emitter_sym,
        on: on_sym,
        emit: emit_sym,
        ..
    } = state_mut(sc).sym;

    let event_emitter_prototype = {
        let event_emitter_prototype = EventEmitter {
            object: NamedObject::new(sc),
            handlers: RefCell::new(FxHashMap::default()),
        };
        let on_fn = register_native_fn(sc, on_sym, on);
        event_emitter_prototype.set_property(sc, on_sym.into(), PropertyValue::static_default(on_fn.into()))?;
        let emit_fn = register_native_fn(sc, emit_sym, emit);
        event_emitter_prototype.set_property(sc, emit_sym.into(), PropertyValue::static_default(emit_fn.into()))?;
        sc.register(event_emitter_prototype)
    };

    let event_emitter_ctor = {
        let event_emitter_ctor = Function::new(
            sc,
            Some(event_emitter_sym.into()),
            FunctionKind::Native(|cx| {
                let EventsState {
                    event_emitter_prototype,
                    event_emitter_constructor,
                } = &State::from_vm(cx.scope).store[EventsKey];

                let emitter = EventEmitter {
                    object: NamedObject::with_prototype_and_constructor(
                        event_emitter_prototype.clone(),
                        event_emitter_constructor.clone(),
                    ),
                    handlers: RefCell::new(FxHashMap::default()),
                };
                Ok(cx.scope.register(emitter).into())
            }),
        );
        event_emitter_ctor.set_fn_prototype(event_emitter_prototype.clone());
        sc.register(event_emitter_ctor)
    };

    State::from_vm_mut(sc).store.insert(
        EventsKey,
        EventsState {
            event_emitter_constructor: event_emitter_ctor.clone(),
            event_emitter_prototype,
        },
    );

    event_emitter_ctor.set_property(
        sc,
        event_emitter_sym.into(),
        PropertyValue::static_default(event_emitter_ctor.clone().into()),
    )?;

    Ok(Value::Object(event_emitter_ctor))
}

#[derive(Debug, Trace)]
pub struct EventEmitter {
    object: NamedObject,
    handlers: RefCell<FxHashMap<Symbol, Vec<Handle>>>,
}

struct EventsKey;
impl Key for EventsKey {
    type State = EventsState;
}

#[derive(Debug, Trace)]
struct EventsState {
    event_emitter_prototype: Handle,
    event_emitter_constructor: Handle,
}

impl Object for EventEmitter {
    delegate!(
        object,
        get_own_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        apply,
        as_any,
        own_keys
    );
}

fn on(cx: CallContext) -> Result<Value, Value> {
    let [name, cb] = &*cx.args else {
        throw!(cx.scope, Error, "expected an event name and callback function");
    };
    let name = name.to_js_string(cx.scope)?;
    let Value::Object(cb) = cb else {
        throw!(cx.scope, Error, "expected callback to be a function")
    };
    let Some(this) = cx.this.downcast_ref::<EventEmitter>() else {
        throw!(cx.scope, TypeError, "on can only be called on EventEmitter instances")
    };
    match this.handlers.borrow_mut().entry(name.sym()) {
        Entry::Occupied(mut entry) => entry.get_mut().push(cb.clone()),
        Entry::Vacant(entry) => drop(entry.insert(vec![cb.clone()])),
    };
    Ok(cx.this)
}

fn emit(cx: CallContext) -> Result<Value, Value> {
    let [name, args @ ..] = &*cx.args else {
        throw!(cx.scope, Error, "expected an event name");
    };
    let name = name.to_js_string(cx.scope)?;
    let Some(this) = cx.this.downcast_ref::<EventEmitter>() else {
        throw!(cx.scope, TypeError, "on can only be called on EventEmitter instances")
    };
    let mut did_emit = false;
    if let Some(handlers) = this.handlers.borrow().get(&name.sym()) {
        for handler in handlers {
            handler
                .apply(cx.scope, cx.this.clone(), args.to_owned())
                .root_err(cx.scope)?;
            did_emit = true;
        }
    }

    Ok(Value::Boolean(did_emit))
}
