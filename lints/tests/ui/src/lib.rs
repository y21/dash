#![cfg_attr(dash_lints, feature(register_tool))]
#![cfg_attr(dash_lints, register_tool(dash_lints))]
#![allow(path_statements, dead_code, clippy::no_effect)]

use dash_vm::localscope::LocalScope;
use dash_vm::value::Unrooted;
use dash_vm::Vm;

fn make_unrooted() -> Unrooted {
    todo!()
}
fn make_unrooted_res(_: &mut LocalScope<'_>) -> Result<Unrooted, Unrooted> {
    todo!()
}
fn transform_unrooted(_: &mut LocalScope<'_>, _: Unrooted) -> Unrooted {
    todo!()
}
fn use_vm(_: &mut Vm) {}
fn use_scope(_: &mut LocalScope<'_>) {}
#[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
fn use_scope_ok(_: &mut LocalScope<'_>) {}

fn with_vm(vm: &mut Vm, unrooted: Unrooted) {
    use_vm(vm);
    unrooted; // killed
}

fn with_scope(scope: &mut LocalScope<'_>, unrooted: Unrooted) {
    use_scope(scope);
    unrooted; // killed
}

fn kill_after_fn(scope: &mut LocalScope, u1: Unrooted, u2: Unrooted) {
    let v = transform_unrooted(scope, u1);
    u2; // killed
    v; // live
}

fn controlflow(scope: &mut LocalScope, cond: u8, u1: Unrooted) {
    match cond {
        0 => {
            use_scope(scope);
        }
        1 => {
            std::hint::black_box(());
        }
        _ => {
            std::hint::black_box(());
        }
    } // join = use

    u1; // killed
}

fn try_err(scope: &mut LocalScope<'_>) -> Result<(), Unrooted> {
    let u1 = make_unrooted_res(scope)?;
    let x = &u1;

    let u2 = make_unrooted_res(scope)?;
    x; // killed;
    u1; // killed
    u2;
    Ok(())
}

fn attribute_works(scope: &mut LocalScope<'_>, unrooted: Unrooted) {
    use_scope_ok(scope);
    unrooted;
}
