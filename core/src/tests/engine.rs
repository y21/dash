use crate::{
    compiler::{
        compiler::{Compiler, FunctionKind},
        instruction::to_vm_instructions,
    },
    eval,
    vm::{frame::Frame, VM},
};

macro_rules! assert_eval_num {
    ($left:expr, $right:expr) => {{
        let (result, _vm) = $left.unwrap();
        let result = unsafe { result.unwrap().borrow_unbounded() }.as_number();
        assert_eq!(result, ($right) as f64);
    }};
}

#[test]
pub fn recursion() {
    let result = eval::<()>(
        r#"
    function recurse(a, b) {
        if (a === 0) {
            return b;
        }
    
        return recurse(a - 1, b * 2);
    }
    
    recurse(50, 2);
    "#,
        None,
    );

    assert_eval_num!(result, 2251799813685248f64);
}

#[test]
pub fn tree() {
    // Taken from https://github.com/boa-dev/boa/issues/1183 and modified a bit
    // This caught a strange bug that used to exist in this engine
    let result = eval::<()>(
        r#"
    function Node(left, right) {
        this.left = left;
        this.right = right;
    }

    let nNodes = 0;
    function makeTree(depth) {
        nNodes += 1;
        if (depth == 0) {
            return new Node();
        }
        const na = makeTree(depth - 1);
        const nb = makeTree(depth - 1);
        return new Node(na, nb);
    }
    
    let tree = makeTree(5);
    nNodes
    "#,
        None,
    );

    assert_eval_num!(result, 63f64);
}

#[test]
pub fn leak() {
    let result = eval::<()>(
        r#"
function foo(a, b) {
    if (a === 0) return;
    const b = 254;
    return foo(a - 1, {});
}
foo(5);
63
    "#,
        None,
    );

    assert_eval_num!(result, 63f64);
}

#[test]
pub fn loop_break() {
    let result = eval::<()>(
        r#"
        let i = 0;
        for (;;) {
            if (++i % 2 === 0 && i > 50) {
                break;
            }
        }
        i
    "#,
        None,
    );

    assert_eval_num!(result, 52f64);
}

#[test]
pub fn single_line_comments() {
    let result = eval::<()>(
        r#"
        // hello
        1+2
    "#,
        None,
    );

    assert_eval_num!(result, 3f64);
}

#[test]
pub fn multi_line_comments() {
    let result = eval::<()>(
        r#"
        /*
        this is a comment
        that is spread across
        several
        lines
        */

        /**/

        3+3
    "#,
        None,
    );

    assert_eval_num!(result, 6f64);
}

#[test]
pub fn else_if() {
    let result = eval::<()>(
        r#"if (false) {
            console.log("no");
          } else if (false) {
            console.log("no");
          } else {
              
          }
          let x = 6; x
    "#,
        None,
    );

    assert_eval_num!(result, 6f64);
}

#[test]
pub fn conditional() {
    let result = eval::<()>(
        r#"
        typeof true ? 6 : 1
    "#,
        None,
    );

    assert_eval_num!(result, 6f64);
}

#[test]
pub fn property_lookup_this_binding() {
    let result = eval::<()>(
        r#"
        true.constructor === Boolean ? 6 : false
    "#,
        None,
    );

    assert_eval_num!(result, 6f64);
}

#[test]
pub fn if_() {
    // Tests for miss-compilation in if statements
    // e.g. invalid jumps or other index errors
    eval::<()>(
        r#"
        function assert_eq(lhs, rhs) {
            if (lhs !== rhs) {
                throw new Error("FAIL");
            }
        }

        function fail() {
            throw new Error("unreachable code detected");
        }

        if (false) { fail(); }
        else if (false) { fail(); }
        else { 1+1 };

        assert_eq(6 * 6 - 35, 1);

        if (true) {}

        assert_eq(3 * 3 - 7, 2);

        if (false) { fail(); } else if (false) { fail(); } else {}
        if (false) { fail(); } else if (false) { fail(); }
        if (true) {} else { fail(); }
        if (true) {} else if (true) { fail(); } else { fail(); }
        if (true) {} else if (false) { fail(); } else { fail(); }

        assert_eq(4 * 4 - 14, 2);

    "#,
        None,
    )
    .unwrap();
}

#[test]
pub fn stack_reset() {
    eval::<()>(
        r#"
        function f() {}
        f([1].map(x => x));
    "#,
        None,
    )
    .unwrap();
}

#[test]
pub fn generator() {
    eval::<()>(
        r#"
        function assert(l, r) {
            if (l !== r) {
                throw new Error("FAIL");
            }
        }

        function* powersOf(n) {
            yield n ** 0;
            yield n ** 1;
            yield n ** 2;
            yield n ** 3;
        }

        const generator = powersOf(2);
        assert(generator.next().value, 1);
        assert(generator.next().value, 2);
        assert(generator.next().value, 4);
        assert(generator.next().value, 8);
        assert(generator.next().value, undefined);
    "#,
        None,
    )
    .unwrap();
}

#[test]
pub fn generator_loop() {
    eval::<()>(
        r#"
        function assert(l, r) {
            if (l !== r) {
                throw new Error("FAIL");
            }
        }

        function* powersOf(n) {
            for (let i = 0; ; ++i) {

                // A bunch of locals...
                let z = 1;
                let zz = 2;
                let zzz = 3;

                yield n ** i;
            }
        }
        
        const generator = powersOf(2);
        assert(generator.next().value, 1);
        assert(generator.next().value, 2);
        assert(generator.next().value, 4);
        assert(generator.next().value, 8);
        assert(generator.next().value, 16);
        assert(generator.next().value, 32);
    "#,
        None,
    )
    .unwrap();
}

#[test]
pub fn in_keyword() {
    eval::<()>(
        r#"
        function assert(l, r) {
            if (l !== r) {
                throw new Error("FAIL");
            }
        }

        assert('a' in { a: 1 }, true);
        assert('a' in {}, false);
        assert('toString' in {}, true);
        "#,
        None,
    )
    .unwrap();
}

#[test]
pub fn instanceof_keyword() {
    eval::<()>(
        r#"
        function assert(l, r) {
            if (l !== r) {
                throw new Error("FAIL");
            }
        }

        function Obj() {}
        let o = new Obj();

        assert({} instanceof Object, true);
        assert(o instanceof Obj, true);
        "#,
        None,
    )
    .unwrap();
}

#[test]
pub fn async_task() {
    let mut vm = VM::from_str::<()>(
        r#"
        console.log("hi");
        123
    "#,
        None,
    )
    .unwrap();

    let (buffer, constants, _gc) = Compiler::<()>::from_str(
        r#"
        console.log("async task?!");
    "#,
        None,
        FunctionKind::Function,
    )
    .unwrap()
    .compile()
    .unwrap();

    let buffer = to_vm_instructions(buffer);

    let frame = Frame::from_buffer(buffer, constants, &vm);

    vm.queue_async_task(frame);

    vm.interpret().unwrap().unwrap();

    vm.run_async_tasks();
}
