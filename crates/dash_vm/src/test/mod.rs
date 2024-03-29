use std::ptr;

use dash_middle::interner::sym;
use dash_optimizer::OptLevel;

use crate::gc::persistent::Persistent;
use crate::value::object::{NamedObject, Object, PropertyValue};
use crate::value::primitive::Number;
use crate::value::{Root, Value};
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
    let value = array
        .get_property(&mut scope, sym::zero.into())
        .unwrap()
        .root(&mut scope);
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
        let dummy_string = scope.register(NamedObject::null());
        let object = NamedObject::new(&scope);
        let key = scope.intern("foo");
        object
            .set_property(
                &mut scope,
                key.into(),
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
    assert!(ptr::eq(
        vm.external_refs.iter().next().unwrap().as_erased_ptr(),
        object.as_erased_ptr()
    ));
    drop(p2);
    assert_eq!(p1.refcount(), 1);
    assert!(vm.external_refs.len() == 1);
    vm.perform_gc();
    assert_eq!(p1.refcount(), 1);
    assert!(vm.external_refs.len() == 1);

    // Check that p1 and object are still alive after GC.
    let mut scope = vm.scope();
    let key = scope.intern("foo");
    let p = p1.get_property(&mut scope, key.into()).unwrap().root(&mut scope);
    assert!(p.downcast_ref::<NamedObject>().is_some());
}

#[test]
fn async_tasks() {
    let mut vm = Vm::new(Default::default());
    vm.eval(
        r#"
    (async function() {
        for (let i = 0,j = 0; i < 5; i++, j++) {
            const res = await Promise.resolve(i);
            if (res != j) {
                throw "Promise resolved to wrong value";
            }
        }
    })();
    "#,
        Default::default(),
    )
    .unwrap();
    assert!(vm.async_tasks.len() == 1);
    vm.perform_gc();
    vm.process_async_tasks();
    assert!(vm.async_tasks.is_empty());
    assert!(vm.stack.is_empty());
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

simple_test!(
    loops,
    r#"
    let sum = 0;
    for (let i = 0; i < 10; i++) {
        sum += i;
    }
    assert(sum === 45);

    sum = 0;
    for (let i = 10; i > 0; i--) {
        sum += i;
    }
    assert(sum === 55);

    sum = 0;
    for (let i = 0; i < 10; i += 2) {
        sum += i;
    }
    assert(sum === 20);

    sum = 0;
    for (let i = 0; i < 10; i++) {
        if (i === 5) {
            continue;
        }
        sum += i;
    }
    assert(sum === 40);

    sum = 0;
    for (let i = 0; i < 10; i++) {
        if (i === 5) {
            break;
        }
        sum += i;
    }
    assert(sum === 10);

    sum = 0;
    for (; false; ) {
        sum += 1;
    }
    assert(sum === 0);

    sum = 0;
    let i = 0;
    for (; i < 10; ) {
        sum += i;
        i++;
    }
    assert(sum === 45);

    sum = 0;
    i = 0;
    for (; ; ) {
        if (i >= 10) {
            break;
        }
        sum += i;
        i++;
    }
    assert(sum === 45);

    sum = 0;
    i = 0;
    while (i < 10) {
        sum += i;
        i++;
    }
    assert(sum === 45);

    "#,
    Value::undefined()
);

simple_test!(
    ternary_assignment_same_level,
    r#"
    let x = {};
    true ? x.a = 1 : x.b = 2;
    assert(x.a === 1);
    assert(x.b === undefined);
    "#,
    Value::undefined()
);

simple_test!(
    sequence_precedence,
    r#"
    let x = [...[1,2],[3,4]];
    assert(x[0] === 1);
    assert(x[1] === 2);
    assert(x[2][0] === 3);
    assert(x[2][1] === 4);
    let o = {a:1,b:2,...{c:3,d:4},e:5};
    
    assert(o.a === 1);
    assert(o.b === 2);
    assert(o.c === 3);
    assert(o.d === 4);
    assert(o.e === 5);
    assert(((a, b) => 1, 2) === 2);
    let sum = (a, b, c, d) => a+b+c+d;
    assert(sum(...[1, 2, ...[3, 4]]) === 10);

    switch (2) {
        case 1, 2: 'PASS'; break;
        default: throw 'FAIL';
    }

    let v = (1, 2, 3);
    let x1 = 1, x2 = 2, x3;
    assert(v === 3);
    assert(x1 == 1);
    assert(x2 == 2);
    assert(x3 === undefined);
    assert((() => 1+2)() === 3);
    "#,
    Value::undefined()
);

simple_test!(
    spread_operator,
    r#"
    function* generator() {
        let arr = [...arguments];
        assert(arr.length === 5);
        [0, 1, 2, 3, 4].forEach((v, i) => assert(arr[i] === v));
    }
    generator(0, ...[1, 2, 3], 4).next();
    
    assert(
        ((...x) => x.length)(1) === 1
            && ((...x) => x.length)(1, 2) === 2
            && ((...x) => x.length)(1, 2, 3) === 3
            && ((...x) => x.length)(...[1, 2, 3]) === 3
            && ((w, ...x) => x.length)(1, 2, 3) === 2
            && ((v, w, ...x) => x.length)(1, 2, 3) === 1
    );
    assert([...[1, 2, 3]].length === 3);
    assert([...[1]].length === 1);
    "#,
    Value::undefined()
);

simple_test!(
    externals,
    r#"
    let x = 1;
    let y = 2;
    let z = 3;

    (function() {
        assert(x === 1);
        assert(y === 2);
        (function() {
            assert(y === 2);
            assert(z === 3);
            (function() {
                assert(x === 1);
                assert(y === 2);
                assert(z === 3);
            })();
        })();
    })();
    
    (function() {
        (function() {
            x = [1, 2];
            x.map(() => y = 5);
        })();
        
        assert(y === 5);
    })();

    let getX = () => x;
    getX().push(3);
    assert(getX().toString() === '1,2,3');
    (function() {
        x = 4;
    })();
    assert(getX() === 4);
    "#,
    Value::undefined()
);

simple_test!(
    holey_array_literal,
    r#"
assert([,].length === 1);
assert([1,].length === 1);
assert([1,,].length === 2);
assert([,,].length === 2);
"#,
    Value::undefined()
);

simple_test!(
    function_apply,
    r#"
function sum(...args) { return args.reduce((p,c)=>p+c,0); }
assert(sum.apply(null, [1,2,3]) === 6);
assert(sum.apply(null) === 0);
assert(sum.apply(null, null) === 0);
assert(sum.apply(null, 0) === 0);
    "#,
    Value::undefined()
);
