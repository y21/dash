use dash_optimizer::OptLevel;

use crate::value::primitive::Number;
use crate::value::Value;
use crate::Vm;

const INTERPRETER: &str = include_str!("interpreter.js");

#[cfg(not(miri))] // miri is too slow for this :(
#[test]
fn interpreter() {
    let mut vm = Vm::new(Default::default());
    let mut scope = vm.scope();
    let value = scope.eval(INTERPRETER, OptLevel::Basic).unwrap().root(&mut scope);

    assert_eq!(scope.stack.len(), 0);
    assert_eq!(scope.frames.len(), 0);
    match value {
        Value::Number(Number(n)) => assert_eq!(n, 1275.0),
        other => unreachable!("{:?}", other),
    }
}

// A test that is simple enough for miri to run in a finite amount of time and to test all of the internals (allocating objects, GCing)
#[test]
fn simple() {
    let mut vm = Vm::new(Default::default());
    let mut scope = vm.scope();
    scope
        .eval(
            r#"
        globalThis.x = [];
        for (let i = 0; i < 4; i++) x.push({i});
    "#,
            OptLevel::Basic,
        )
        .unwrap();
    scope.perform_gc();
    let array = scope
        .eval("[x[0].i + x[1].i + x[2].i + x[3].i]", OptLevel::Basic)
        .unwrap()
        .root(&mut scope);
    scope.perform_gc();
    let value = array.get_property(&mut scope, "0".into()).unwrap();
    assert_eq!(scope.stack.len(), 0);
    assert_eq!(scope.frames.len(), 0);
    assert_eq!(value, Value::number(6.0));
}
