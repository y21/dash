use dash_optimizer::OptLevel;

use crate::value::primitive::Number;
use crate::value::Value;
use crate::Vm;

const INTERPRETER: &str = include_str!("interpreter.js");

#[cfg(not(miri))] // miri is too slow for this :(
#[test]
fn interpreter() {
    let mut vm = Vm::new(Default::default());
    let value = vm.eval(INTERPRETER, OptLevel::Basic).unwrap();

    assert_eq!(vm.stack.len(), 0);
    assert_eq!(vm.frames.len(), 0);
    match value {
        Value::Number(Number(n)) => assert_eq!(n, 1275.0),
        _ => unreachable!("{:?}", value),
    }
}

// A test that is simple enough for miri to run in a finite amount of time and to test all of the internals (allocating objects, GCing)
#[test]
fn simple() {
    let mut vm = Vm::new(Default::default());
    vm.eval(
        r#"
        globalThis.x = [];
        for (let i = 0; i < 4; i++) x.push({i});
    "#,
        OptLevel::Basic,
    )
    .unwrap();
    vm.perform_gc();
    let value = vm.eval("x[0].i + x[1].i + x[2].i + x[3].i", OptLevel::Basic).unwrap();
    assert_eq!(vm.stack.len(), 0);
    assert_eq!(vm.frames.len(), 0);
    assert_eq!(value, Value::number(6.0));
}
