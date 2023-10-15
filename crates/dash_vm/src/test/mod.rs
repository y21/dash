use std::rc::Rc;

use dash_optimizer::OptLevel;

use crate::gc::persistent::Persistent;
use crate::value::object::NamedObject;
use crate::value::object::Object;
use crate::value::object::PropertyValue;
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
    // Try nesting scopes, even more than the default
    let mut scope = scope.scope();
    let mut scope = scope.scope();
    let mut scope = scope.scope();
    let mut scope = scope.scope();
    let mut scope = scope.scope();
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

#[test]
fn persistent_trace() {
    // This has caused issues in the past. Essentially,
    // `Persistent<T>` is refcounted, but it used to not be traced,
    // so its reachables could still be deallocated.

    let mut vm = Vm::new(Default::default());
    let object = {
        let mut scope = vm.scope();
        let dummy_string = scope.register(Rc::<str>::from("hi"));
        let object = NamedObject::new(&scope);
        object
            .set_property(
                &mut scope,
                "foo".into(),
                PropertyValue::static_default(Value::Object(dummy_string)),
            )
            .unwrap();
        scope.register(object)
    }; // scope dropped here

    assert!(vm.external_refs.is_empty());
    let p1 = Persistent::new(&mut vm, object.clone());
    assert_eq!(p1.refcount(), 1);
    assert!(vm.external_refs.len() == 1);
    let p2 = Persistent::new(&mut vm, object.clone());
    assert_eq!(p1.refcount(), 2);
    assert!(vm.external_refs.len() == 1);
    assert!(vm.external_refs.iter().next().unwrap().as_ptr() == object.as_ptr());
    drop(p2);
    assert_eq!(p1.refcount(), 1);
    assert!(vm.external_refs.len() == 1);
    vm.perform_gc();
    assert_eq!(p1.refcount(), 1);
    assert!(vm.external_refs.len() == 1);

    // Check that p1 and object are still alive after GC.
    let p = p1.get_property(&mut vm.scope(), "foo".into()).unwrap();
    assert_eq!(p.downcast_ref::<Rc<str>>().unwrap().as_ref(), "hi");
}

macro_rules! simple_test {
    ($testname:ident, $code:expr, $expected:expr) => {
        #[test]
        fn $testname() {
            let mut vm = Vm::new(Default::default());
            let scope = &mut vm.scope();
            let prelude = String::from(
                r"
            function assert(c, e) {
                if (!c) {
                    throw e;
                }
            }
            ",
            );
            let value = scope
                .eval(&(prelude + $code), Default::default())
                .unwrap()
                .root(scope);
            assert_eq!(value, $expected);
            assert!(scope.stack.is_empty());
        }
    };
}

simple_test!(
    spread_argument_position,
    r"
function x(...v) {
    assert(v.length === 12, 'Expected array length 12, got: ' + v.length);
    for (let i = 0; i < 12; i++)
        assert(v[i] === i, `Expected v[${i}] == ${i}, got: ${v[i]}`);
}

x(0,...[1,2],3,4,...[5,6],7,8,9,10,...[11]);",
    Value::undefined()
);

simple_test!(
    error_structure,
    r#"
    assert(new ReferenceError().constructor === ReferenceError);
    assert(new ReferenceError().__proto__ === ReferenceError.prototype);
    assert(new ReferenceError().__proto__.__proto__ === Error.prototype);
    assert(new Error("foo").message === "foo");
    assert(new ReferenceError("foo").toString().startsWith("ReferenceError: foo"));
    assert(new Error("foo").toString().startsWith("Error: foo"));
    "#,
    Value::undefined()
);
