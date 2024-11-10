use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::ops::ControlFlow;

use dash_middle::interner::Symbol;
use dash_proc_macro::Trace;
use dash_rt::state::State;
use dash_rt::typemap::Key;
use dash_vm::gc::ObjectId;
use dash_vm::localscope::LocalScope;
use dash_vm::value::function::native::{register_native_fn, CallContext};
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::root_ext::RootErrExt;
use dash_vm::value::{Unpack, Value, ValueKind};
use dash_vm::{delegate, throw};
use rustc_hash::FxHashMap;

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

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
                } = State::from_vm(cx.scope).store[EventsKey];

                let emitter = EventEmitter {
                    object: NamedObject::with_prototype_and_constructor(
                        event_emitter_prototype,
                        event_emitter_constructor,
                    ),
                    handlers: RefCell::new(FxHashMap::default()),
                };
                Ok(cx.scope.register(emitter).into())
            }),
        );
        event_emitter_ctor.set_fn_prototype(event_emitter_prototype);
        sc.register(event_emitter_ctor)
    };

    State::from_vm_mut(sc).store.insert(
        EventsKey,
        EventsState {
            event_emitter_constructor: event_emitter_ctor,
            event_emitter_prototype,
        },
    );

    event_emitter_ctor.set_property(
        sc,
        event_emitter_sym.into(),
        PropertyValue::static_default(event_emitter_ctor.into()),
    )?;

    Ok(Value::object(event_emitter_ctor))
}

#[derive(Debug, Trace)]
pub struct EventEmitter {
    object: NamedObject,
    handlers: RefCell<FxHashMap<Symbol, Vec<ObjectId>>>,
}

struct EventsKey;
impl Key for EventsKey {
    type State = EventsState;
}

#[derive(Debug, Trace)]
struct EventsState {
    event_emitter_prototype: ObjectId,
    event_emitter_constructor: ObjectId,
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

fn with_event_emitter(
    sc: &mut LocalScope<'_>,
    v: Value,
    f: impl Fn(&mut LocalScope<'_>, &EventEmitter) -> Result<Value, Value>,
) -> Result<Value, Value> {
    let cf = v.for_each_prototype(sc, |sc, v| {
        if let Some(e) = v.unpack().downcast_ref::<EventEmitter>(sc) {
            Ok(ControlFlow::Break(f(sc, e)?))
        } else {
            Ok(ControlFlow::Continue(()))
        }
    })?;

    match cf {
        ControlFlow::Break(b) => Ok(b),
        ControlFlow::Continue(()) => throw!(sc, TypeError, "Incompatible EventEmitter receiver"),
    }
}

fn on(cx: CallContext) -> Result<Value, Value> {
    let [name, cb] = *cx.args else {
        throw!(cx.scope, Error, "expected an event name and callback function");
    };
    let name = name.to_js_string(cx.scope)?;
    let ValueKind::Object(cb) = cb.unpack() else {
        throw!(cx.scope, Error, "expected callback to be a function")
    };
    with_event_emitter(cx.scope, cx.this, |_, this| {
        match this.handlers.borrow_mut().entry(name.sym()) {
            Entry::Occupied(mut entry) => entry.get_mut().push(cb),
            Entry::Vacant(entry) => drop(entry.insert(vec![cb])),
        };
        Ok(cx.this)
    })
}

fn emit(cx: CallContext) -> Result<Value, Value> {
    let [name, args @ ..] = &*cx.args else {
        throw!(cx.scope, Error, "expected an event name");
    };
    let name = name.to_js_string(cx.scope)?;
    with_event_emitter(cx.scope, cx.this, |sc, this| {
        let mut did_emit = false;
        if let Some(handlers) = this.handlers.borrow().get(&name.sym()) {
            for handler in handlers {
                handler.apply(sc, cx.this, args.to_owned()).root_err(sc)?;
                did_emit = true;
            }
        }

        Ok(Value::boolean(did_emit))
    })
}
