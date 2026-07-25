use std::time::Instant;

use dash_middle::interner::{Symbol, sym};
use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::ops::conversions::ValueConversion;

use crate::state::state_mut;

fn label_from_value(value: Option<&Value>, scope: &mut LocalScope) -> Result<Symbol, Value> {
    Ok(match value {
        Some(v) => v.to_js_string(scope)?.sym(),
        None => sym::default,
    })
}

pub fn console_time(cx: CallContext) -> Result<Value, Value> {
    let label = label_from_value(cx.args.first(), cx.scope)?;

    state_mut(cx.scope).timer_map.insert(label, Instant::now());
    Ok(Value::undefined())
}

pub fn console_time_end(cx: CallContext) -> Result<Value, Value> {
    let label = label_from_value(cx.args.first(), cx.scope)?;

    let start_time = state_mut(cx.scope).timer_map.remove(&label);
    if let Some(start_time) = start_time {
        let elapsed = start_time.elapsed();
        println!("{}: {elapsed:?}", cx.scope.interner.resolve(label))
    } else {
        println!("Warning: No timer found for label '{}'", label);
    }

    Ok(Value::undefined())
}
