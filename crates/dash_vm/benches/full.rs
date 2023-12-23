use criterion::{criterion_group, criterion_main, Criterion};
use dash_optimizer::OptLevel;

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
            let mut vm = dash_vm::Vm::new(Default::default());
            vm.eval(CODE, OptLevel::Aggressive).unwrap();
        })
    });
    cr.bench_function("fib_recursive(12)", |b| {
        b.iter(|| {
            let mut vm = dash_vm::Vm::new(Default::default());
            vm.eval(FIBONACCI_RECURSIVE, OptLevel::Aggressive).unwrap();
        })
    });
    cr.bench_function("fib_iterative(12)", |b| {
        b.iter(|| {
            let mut vm = dash_vm::Vm::new(Default::default());
            vm.eval(FIBONACCI_ITERATIVE, OptLevel::Aggressive).unwrap();
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
