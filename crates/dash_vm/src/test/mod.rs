use core::f64;
use std::ptr;

use dash_middle::interner::sym;

use crate::Vm;
use crate::gc::ObjectId;
use crate::gc::persistent::Persistent;
use crate::value::object::{Object, OrdObject, PropertyValue};
use crate::value::primitive::{Null, Number, Symbol, Undefined};
use crate::value::propertykey::ToPropertyKey;
use crate::value::{Root, Unpack, Value, ValueKind};

const INTERPRETER: &str = include_str!("interpreter.js");

#[cfg(not(miri))] // miri is too slow for this :(
#[test]
fn interpreter() {
    let mut vm = Vm::new(Default::default());
    let mut scope = vm.scope();
    let value = scope.eval(INTERPRETER).unwrap().root(&mut scope);

    assert_eq!(scope.stack.len(), 0);
    assert_eq!(scope.frames.len(), 0);
    match value.unpack() {
        ValueKind::Number(Number(n)) => assert_eq!(n, 1275.0),
        other => unreachable!("{:?}", other),
    }
}

#[test]
fn packed_value() {
    assert_eq!(Value::null().unpack(), ValueKind::Null(Null));
    assert_eq!(Value::undefined().unpack(), ValueKind::Undefined(Undefined));
    assert_eq!(Value::boolean(true).unpack(), ValueKind::Boolean(true));
    assert_eq!(Value::boolean(false).unpack(), ValueKind::Boolean(false));
    assert!(matches!(
        Value::external(ObjectId::from_raw(u32::MAX)).unpack(),
        ValueKind::External(ext) if ext.id() == ObjectId::from_raw(u32::MAX)
    ));
    assert!(matches!(
        Value::external(ObjectId::from_raw(4242)).unpack(),
        ValueKind::External(ext) if ext.id() == ObjectId::from_raw(4242)
    ));
    assert_eq!(Value::number(0.0).unpack(), ValueKind::Number(Number(0.0)));
    match Value::number(f64::NAN).unpack() {
        ValueKind::Number(num) => assert!(num.0.is_nan()),
        other => panic!("wrong type: {other:?}"),
    }
    assert_eq!(
        Value::number(f64::INFINITY).unpack(),
        ValueKind::Number(Number(f64::INFINITY))
    );
    assert_eq!(
        Value::number(f64::NEG_INFINITY).unpack(),
        ValueKind::Number(Number(f64::NEG_INFINITY))
    );
    assert_eq!(
        Value::object(ObjectId::from_raw(u32::MAX)).unpack(),
        ValueKind::Object(ObjectId::from_raw(u32::MAX))
    );
    assert_eq!(
        Value::object(ObjectId::from_raw(4242)).unpack(),
        ValueKind::Object(ObjectId::from_raw(4242))
    );
    assert_eq!(
        Value::string(sym::Array.into()).unpack(),
        ValueKind::String(sym::Array.into())
    );
    assert_eq!(
        Value::symbol(Symbol::new(sym::Array.into())).unpack(),
        ValueKind::Symbol(Symbol::new(sym::Array.into()))
    );

    let reprs = [
        Value::BOOLEAN_MASK,
        Value::NULL_MASK,
        Value::OBJECT_MASK,
        Value::STRING_MASK,
        Value::SYMBOL_MASK,
        Value::EXTERNAL_MASK,
        Value::UNDEFINED_MASK,
    ];
    for repr in reprs {
        assert_eq!(
            reprs.iter().filter(|&&c| c == repr).count(),
            1,
            "pattern {repr} must be unique"
        );
        #[expect(clippy::unusual_byte_groupings)]
        let mask: u64 = 0b0_11111111111_11 << (64 - 14);
        assert!(repr & mask == mask);
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
        )
        .unwrap();
    scope.perform_gc();
    let array = scope
        .eval("[x[0].i + x[1].i + x[2].i + x[3].i]")
        .unwrap()
        .root(&mut scope);
    scope.perform_gc();
    let value = array
        .get_property(sym::zero.to_key(&mut scope), &mut scope)
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
        let dummy_string = scope.register(OrdObject::null());
        let object = OrdObject::new(&scope);
        let key = scope.intern("foo");
        object
            .set_property(
                key.to_key(&mut scope),
                PropertyValue::static_default(Value::object(dummy_string)),
                &mut scope,
            )
            .unwrap();
        scope.register(object)
    }; // scope dropped here

    assert!(vm.external_refs.0.borrow().is_empty());
    let p1 = Persistent::new(&mut vm, object);
    assert_eq!(p1.refcount(), 1);
    assert!(vm.external_refs.0.borrow().len() == 1);
    let p2 = Persistent::new(&mut vm, object);
    assert_eq!(p1.refcount(), 2);
    assert!(vm.external_refs.0.borrow().len() == 1);
    assert!(ptr::eq(
        vm.external_refs.0.borrow().keys().next().unwrap().data_ptr(&vm),
        object.data_ptr(&vm)
    ));
    drop(p2);
    assert_eq!(p1.refcount(), 1);
    assert!(vm.external_refs.0.borrow().len() == 1);
    vm.perform_gc();
    assert_eq!(p1.refcount(), 1);
    assert!(vm.external_refs.0.borrow().len() == 1);

    // Check that p1 and object are still alive after GC.
    let mut scope = vm.scope();
    let key = scope.intern("foo");
    let p = p1
        .get_property(key.to_key(&mut scope), &mut scope)
        .unwrap()
        .root(&mut scope);
    assert!(p.unpack().downcast_ref::<OrdObject>(&scope).is_some());
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
            let value = scope.eval(&(prelude + $code)).unwrap().root(scope);
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

x(0,...[1,2],3,4,...[5,6],7,...[],8,9,10,...[11], ...[], ...[], ...[]);",
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

simple_test!(
    classes,
    r#"
    class C1 {}
    new C1();
    
    class C2 { field = sideEffect() }
    /* don't evaluate C2 constructor */
    
    class C3 { field = 4 * 4 }
    assert('field' in new C3());
    assert(new C3().field === 16);
    
    let c = 0;
    class C4 {
        get field() { return c++ };
        // FIXME: uncomment once we have setters
        set field(v) { c = v; }
    }
    assert(new C4().field === 0);
    assert(new C4().field === 1);
    // new C4().field = 0;
    // assert(new C4().field === 0);
    // assert(new C4().field === 1);
    
    class C5 { static field = 42; }
    assert(C5.field === 42);
    assert(!('field' in new C5()));
    
    class C6 { foo(n) { return n * n; }; static bar() { return 42 } }
    assert(new C6().foo(4) === 16);
    assert(C6.bar() === 42);

    class C7 {
        *[Symbol.iterator]() {
            yield 42;
        }
    }
    const gen = new C7()[Symbol.iterator]();
    assert(gen.next().value === 42);
    assert(gen.next().done);

    class C8 {
        get() { return 1 }
        set() { return 2 }
        get a() { return 3 }
    }
    assert(new C8().get() === 1);
    assert(new C8().set() === 2);
    assert(new C8().a === 3);

    assert(new (class { constructor() { return [42] } })()[0] == 42);
    let v = class V {};
    assert(v === V);

    // Issue #101: evaluate subclass member initializers after super() call
    class C9 {}
    class C10 extends C9 {
        v = 1;
        constructor() {
            super();
        }
    }
    assert((new C10()).v === 1);
    "#,
    Value::undefined()
);

simple_test!(
    try_finally,
    r#"

    let o = '';
    assert((() => {
        o += 1;
        try {
            o += 2;
            return 1;
        } catch {
            o += 4;
        } finally {
            o += 3;
        }
    })() == 1);
    assert(o === '123');


    o = '';
    assert((() => {
        o += 1;
        try {
            o += 2;
            return 1;
        } catch {
            o += 4;
        } finally {
            o += 3;
            return 2;
        }
    })() == 2);
    assert(o === '123');


    o = '';
    try {
        o += 1;
        try {
            o += 2;
            throw null;
        } catch {
            o += 3;
        } finally {
            o += 4;
        }
    } catch(e) {
        // exception was already caught by the inner catch
        o += 5;
    } finally {
        o += 6;
    }
    assert(o === '12346');


    o = '';
    try {
        o += 1;
        try {
            o += 2;
            throw null;
        } catch(e) {
            o += 3;
            throw e;
        } finally {
            o += 4;
        }
    } catch(e) {
        // inner exception rethrown, we should catch it here
        assert(e === null);
        o += 5;
    } finally {
        o += 6;
    }
    assert(o === '123456');

    o = '';
    assert((() => {
        try {
            try {
                o += 1;
                return 1;
            } catch(e) {
                o += 2;
            } finally {
                o += 3;
                // DONT return here just yet
            }
            o += 4;
        } finally {
            o += 5;
            return 5;
        }
    })() === 5);
    assert(o === '135');

    // Issue #87
    try {
        try {
            throw 1;
        } catch(e) { }
    
        throw 1;
    } catch(e) { }

    "#,
    Value::undefined()
);

// Issue #89
simple_test!(closure_default_param1, "((v = 1) => v)()", Value::number(1.));
simple_test!(closure_default_param2, "((v = 1) => v)(2)", Value::number(2.));

simple_test!(
    holey_array_join,
    "assert(new Array(6).join('1') === '11111')",
    Value::undefined()
);

simple_test!(
    return_automatic_semicolon_insertion,
    "(function() { return\n5 })()",
    Value::undefined()
);

simple_test!(
    labels,
    r"
    let order = [];
    let i = 0;
    a: while (true) {
        b: while(true) {
            order.push(`i${i}`);
            if (i >= 5) break b;
            i++;
        }
        order.push(`o${i}`);
        if (i == 10) break a;
        i++;
    }
    assert(order == 'i0,i1,i2,i3,i4,i5,o5,i6,o6,i7,o7,i8,o8,i9,o9,i10,o10');

    order = [];
    b: {
        order.push(1);
        break b;
        order.push(2);
    }
    
    b: {
        order.push(3);
        break b;
        order.push(4);
    }
    order.push(5);
    assert(order == '1,3,5' && order != '135');
    ",
    Value::undefined()
);

simple_test!(
    new_target,
    "
    class Sup extends ReferenceError {
        constructor() {
            assert(new.target.name === 'Sub');
            assert(new.target === Sub);
            super();
        }
    }
    class Sub extends Sup {}
    assert(new Sub() instanceof Sub);
    assert(new Sub() instanceof Sup);
    assert(new Sub() instanceof ReferenceError);
    assert(new Sub() instanceof Error);
    ",
    Value::undefined()
);

simple_test!(
    try_after_generator_yield,
    "
    // Issue #96
    function* gen() {
        try {
            yield 0;
            throw 1;
        } catch {}
    }
    let x = gen();
    (() => assert(x.next().value === 0))();
    assert(x.next().done);
    ",
    Value::undefined()
);

// JIT tests

macro_rules! jit_test {
    ($name:ident, $code:expr) => {
        #[cfg(feature = "jit")]
        simple_test!($name, $code, Value::undefined());
    };
}

jit_test!(
    jit_loop_break,
    "
    let i = 0;
    for (; i < 1000; i++) {}
    assert(i === 1000);
    "
);

jit_test!(
    jit_numeric_operators,
    "
    let result;
    for (let i = 0; i < 300; i++) {
        let add = i + 3;
        let sub = i - 3;
        let mul = i * 2;
        let div = i / 2;
        let rem = i % 7;
        let pow = 2 ** (i % 4);
        let or = i | 1;
        let xor = i ^ 15;
        let and = i & 255;
        let shl = (i % 8) << 2;
        let shr = (i % 32) >> 1;
        let ushr = (i % 32) >>> 1;
        let neg = -i;
        let pos = +i;
        let bitnot = ~i;
        result = [add, sub, mul, div, rem, pow, or, xor, and, shl, shr, ushr, neg, pos, bitnot];
    }
    assert(result[0] === 302 && result[1] === 296);
    assert(result[2] === 598 && result[3] === 149.5);
    assert(result[4] === 5 && result[5] === 8);
    assert(result[6] === 299 && result[7] === (299 ^ 15) && result[8] === 43);
    assert(result[9] === 12 && result[10] === 5 && result[11] === 5);
    assert(result[12] === -299 && result[13] === 299 && result[14] === -300);
    "
);

jit_test!(
    jit_comparisons_and_compound_assignments,
    "
    let comparisons;
    let updated;
    for (let i = 0; i < 300; i++) {
        comparisons = [
            i > 100, i >= 299, i < 300, i <= 299,
            i == 299, i != 298, i === 299, i !== 298,
            !i, typeof i === 'number'
        ];

        let n = i;
        n += 2;
        n -= 1;
        n *= 2;
        n /= 2;
        n %= 10;
        n **= 1;
        n <<= 1;
        n >>= 1;
        n >>>= 1;
        n |= 1;
        n ^= 3;
        n &= 255;
        n++;
        ++n;
        n--;
        --n;
        updated = n;
    }
    assert(comparisons[0] && comparisons[1] && comparisons[2] && comparisons[3]);
    assert(comparisons[4] && comparisons[5] && comparisons[6] && comparisons[7]);
    assert(!comparisons[8] && comparisons[9]);
    assert(updated === 2);
    "
);

jit_test!(
    jit_globals_closures_and_calls,
    "
    let globalCount = 1;
    function twice(value) { return value * 2; }

    function run() {
        let captured = 5;
        let sum = 0;
        function inner() {
            for (let i = 0; i < 300; i++) {
                globalCount += i;
                captured += i;
                sum += globalCount + captured + twice(i);
                if (globalCount > 100000) globalCount = 1;
            }
        }
        inner();
        return [captured, sum];
    }

    let result = run();
    assert(globalCount === 44851);
    assert(result[0] === 44855);
    assert(typeof result[1] === 'number');
    "
);

jit_test!(
    jit_this_literals_types_and_calls,
    "
    function loopWithThis() {
        let values;
        for (let i = 0; i < 300; i++) {
            let value = i + 1;
            this.total += id(value);
            values = [
                typeof value, !value, null, undefined,
                true, false, 'hello', 1.5
            ];
        }
        return values;
    }

    function id(value) { return value; }
    let object = { total: 0, loop: loopWithThis };
    let values = object.loop();
    assert(object.total === 45150);
    assert(values[0] === 'number' && values[1] === false);
    assert(values[2] === null && values[3] === undefined);
    assert(values[4] === true && values[5] === false);
    assert(values[6] === 'hello' && values[7] === 1.5);
    "
);

jit_test!(
    jit_static_and_dynamic_property_updates,
    "
    let object = { staticValue: 0, dynamicValue: 0 };
    for (let i = 0; i < 300; i++) {
        object.staticValue = i;
        object.staticValue += 3;
        object.staticValue -= 1;
        object.staticValue *= 2;
        object.staticValue /= 2;
        object.staticValue %= 7;
        object.staticValue **= 1;
        object.staticValue <<= 1;
        object.staticValue >>= 1;
        object.staticValue >>>= 1;
        object.staticValue |= 1;
        object.staticValue ^= 2;
        object.staticValue &= 1023;
        object.staticValue++;
        ++object.staticValue;
        object.staticValue--;
        --object.staticValue;

        object['dynamicValue'] = i;
        object['dynamicValue'] += 3;
        object['dynamicValue'] -= 1;
        object['dynamicValue'] *= 2;
        object['dynamicValue'] /= 2;
        object['dynamicValue'] %= 7;
        object['dynamicValue'] **= 1;
        object['dynamicValue'] <<= 1;
        object['dynamicValue'] >>= 1;
        object['dynamicValue'] >>>= 1;
        object['dynamicValue'] |= 1;
        object['dynamicValue'] ^= 2;
        object['dynamicValue'] &= 1023;
        object['dynamicValue']++;
        ++object['dynamicValue'];
        object['dynamicValue']--;
        --object['dynamicValue'];
    }
    assert(object.staticValue === 3);
    assert(object.dynamicValue === 3);
    "
);

jit_test!(
    jit_debugger_and_exception,
    "
    let i = 0;
    let caught;
    try {
        for (; i < 300; i++) {
            debugger;
            if (i === 200) throw 'boom';
        }
    } catch (error) {
        caught = error;
    }
    assert(i === 200);
    assert(caught === 'boom');
    "
);
