use criterion::{criterion_group, criterion_main, Criterion};
use dash_compiler::FunctionCompiler;
use dash_middle::interner::StringInterner;
use dash_optimizer::OptLevel;
use dash_vm::frame::Frame;
use dash_vm::params::VmParams;
use dash_vm::Vm;

const CODE: &str = include_str!("../src/test/interpreter.js");
const FIBONACCI_RECURSIVE: &str = r"
function fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
return fib(12);
";
const FIBONACCI_ITERATIVE: &str = r"
function fib(n) {
    if (n <= 1) return n;
    let fib = 1;
    let prevFib = 1;
    for (let i = 2; i < n; i++) {
        let temp = fib;
        fib += prevFib;
        prevFib = temp;
    }
    return fib;
}
return fib(12);
";

pub fn benchmark(cr: &mut Criterion) {
    cr.bench_function("interpreter", |b| {
        b.iter(|| {
            let mut vm = Vm::new(Default::default());
            vm.eval(CODE, OptLevel::Aggressive).unwrap();
        })
    });
    cr.bench_function("fib_recursive(12)", |b| {
        b.iter(|| {
            let mut vm = Vm::new(Default::default());
            vm.eval(FIBONACCI_RECURSIVE, OptLevel::Aggressive).unwrap();
        })
    });
    cr.bench_function("fib_iterative(12)", |b| {
        b.iter(|| {
            let mut vm = Vm::new(Default::default());
            vm.eval(FIBONACCI_ITERATIVE, OptLevel::Aggressive).unwrap();
        })
    });
    let mut tinycolor2 = ureq::get("https://www.unpkg.com/tinycolor2@1.6.0/cjs/tinycolor.js")
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    tinycolor2.insert_str(0, "let exports = {}; let module = {exports};");
    tinycolor2.push_str(
        "; let tinycolor = module.exports; let x; for (let i = 0; i < 1000; i++) x = tinycolor('#ff0000').toFilter();",
    );
    cr.bench_function("parse+compile tinycolor2", |b| {
        b.iter(|| {
            let interner = &mut StringInterner::new();
            FunctionCompiler::compile_str(interner, &tinycolor2, OptLevel::Aggressive).unwrap()
        });
    });
    let mut interner = StringInterner::new();
    let compile_result = FunctionCompiler::compile_str(&mut interner, &tinycolor2, OptLevel::Aggressive).unwrap();
    cr.bench_function("exec tinycolor2 parse hex+toFilter", |b| {
        b.iter(|| {
            let mut vm = Vm::new(VmParams::new());
            vm.interner = interner.clone();
            vm.execute_frame(Frame::from_compile_result(compile_result.clone()))
                .unwrap()
        });
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
