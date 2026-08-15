use crate::frame::Ip;
use crate::localscope::LocalScope;

pub fn compile_loop_region(scope: &mut LocalScope<'_>, start: Ip, end: Ip) {
    scope.frames.with_current_bytecode(|bytecode| {
        let loop_bytecode = &bytecode[start.0 as usize..end.0 as usize];
    });
}
