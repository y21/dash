use dash_optimizer::OptLevel;

use crate::value::primitive::Number;
use crate::value::Value;
use crate::Vm;

const INTERPRETER: &str = include_str!("interpreter.js");

#[test]
fn interpreter() {
    let mut vm = Vm::new(Default::default());
    let value = vm.eval(INTERPRETER, OptLevel::Basic).unwrap();

    assert_eq!(vm.stack.len(), 0 + 0);
    assert_eq!(vm.frames.len(), 0);
    match value {
        Value::Number(Number(n)) => assert_eq!(n, 1275.0),
        _ => unreachable!("{:?}", value),
    }
}
