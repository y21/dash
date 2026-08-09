use std::cell::Cell;
use std::process::{Command, Stdio};

use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::array::{Array, ArrayIterator};
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::object::{Object, OrdObject, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::{ExceptionContext, Root, Value};

use crate::buffer::Buffer;
use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let exports = OrdObject::new(sc);
    let NodeSymbols {
        spawnSync: spawn_sync_sym,
        ..
    } = state_mut(sc).sym;

    let spawn_sync = register_native_fn(sc, spawn_sync_sym, false, spawn_sync);
    exports.set_property(
        spawn_sync_sym.to_key(sc),
        PropertyValue::static_default(spawn_sync.into()),
        sc,
    )?;

    Ok(sc.register(exports).into())
}

fn spawn_sync(cx: CallContext) -> Result<Value, Value> {
    let NodeSymbols {
        stdio: stdio_sym,
        output: output_sym,
        stdout: stdout_sym,
        stderr: stderr_sym,
        pid: pid_sym,
        status: status_sym,
        ..
    } = state_mut(cx.scope).sym;

    let command = cx
        .args
        .first()
        .or_type_err(cx.scope, "Missing command to spawnSync")?
        .to_js_string(cx.scope)?;
    let args = cx.args.get(1);
    let options = cx.args.get(2);

    let mut command = Command::new(command.res(cx.scope));

    if let Some(args) = args {
        let args_iter = ArrayIterator::new(cx.scope, *args)?;

        while let Some(arg) = args_iter.next(cx.scope).root(cx.scope)? {
            let arg = arg.to_js_string(cx.scope)?;
            command.arg(arg.res(cx.scope));
        }
    }

    if let Some(options) = options {
        let stdio = options
            .get_property(stdio_sym.to_key(cx.scope), cx.scope)
            .root(cx.scope)?
            .into_option();

        if let Some(stdio) = stdio {
            match stdio.to_js_string(cx.scope)?.res(cx.scope) {
                "inherit" => {
                    command.stdin(Stdio::inherit());
                    command.stdout(Stdio::inherit());
                    command.stderr(Stdio::inherit());
                }
                other => {
                    let other = other.to_string();
                    throw!(cx.scope, Error, "Invalid stdio option passed to spawnSync: {}", other);
                }
            }
        }
    }

    let process = match command.spawn() {
        Ok(c) => c,
        Err(err) => throw!(cx.scope, Error, "Failed to spawn process: {}", err),
    };
    let pid = process.id();

    let output = match process.wait_with_output() {
        Ok(output) => output,
        Err(err) => throw!(cx.scope, Error, "Failed to wait for process output: {}", err),
    };
    let status = output.status.code();

    let stdout = Buffer::from_storage(output.stdout.into_iter().map(Cell::new).collect::<Vec<_>>(), cx.scope);
    let stdout = cx.scope.register(stdout);
    let stderr = Buffer::from_storage(output.stderr.into_iter().map(Cell::new).collect::<Vec<_>>(), cx.scope);
    let stderr = cx.scope.register(stderr);

    let output = Array::from_vec(
        vec![
            PropertyValue::static_default(Value::null()),
            PropertyValue::static_default(Value::object(stdout)),
            PropertyValue::static_default(Value::object(stderr)),
        ],
        cx.scope,
    );
    let output = cx.scope.register(output);

    let result = OrdObject::new(cx.scope);
    result.set_property(
        output_sym.to_key(cx.scope),
        PropertyValue::static_default(Value::object(output)),
        cx.scope,
    )?;
    result.set_property(
        stdout_sym.to_key(cx.scope),
        PropertyValue::static_default(Value::object(stdout)),
        cx.scope,
    )?;
    result.set_property(
        stderr_sym.to_key(cx.scope),
        PropertyValue::static_default(Value::object(stderr)),
        cx.scope,
    )?;
    result.set_property(
        pid_sym.to_key(cx.scope),
        PropertyValue::static_default(Value::number(pid as f64)),
        cx.scope,
    )?;

    result.set_property(
        status_sym.to_key(cx.scope),
        PropertyValue::static_default(match status {
            Some(status) => Value::number(status as f64),
            None => Value::null(),
        }),
        cx.scope,
    )?;

    Ok(cx.scope.register(result).into())
}
