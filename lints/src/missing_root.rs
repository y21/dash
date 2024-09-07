use std::iter;

use clippy_utils::def_path_res;
use mir::visit::MirVisitable as _;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::intravisit::FnKind;
use rustc_hir::{self as hir};
use rustc_index::bit_set::BitSet;
use rustc_index::IndexVec;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::{Obligation, ObligationCause};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::mir::{self, BasicBlock, Body, Local};
use rustc_middle::ty::{ParamEnv, Ty, TyCtxt};
use rustc_middle::{bug, ty};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::Span;
use rustc_trait_selection::traits::ObligationCtxt;

declare_lint! {
    pub MISSING_ROOT,
    Deny,
    "missing .root() calls"
}

#[derive(Default)]
pub struct MissingRoot {
    root: Option<DefId>,
    scope: Option<DefId>,
    vm: Option<DefId>,
}

impl_lint_pass!(MissingRoot => [MISSING_ROOT]);

impl MissingRoot {
    fn root_trait(&self) -> DefId {
        self.root.unwrap()
    }

    fn is_vm_like(&self, ty: Ty<'_>) -> bool {
        if let ty::Adt(def, _) = ty.peel_refs().kind() {
            [self.root.unwrap(), self.scope.unwrap()].contains(&def.did())
        } else {
            false
        }
    }
}

fn unique_res(cx: &LateContext<'_>, path: &[&str], kind: DefKind) -> Option<DefId> {
    let mut unrooted_res = def_path_res(cx.tcx, path).into_iter();
    if let Some(Res::Def(res_kind, def_id)) = unrooted_res.next()
        && res_kind == kind
        && unrooted_res.next().is_none()
    {
        Some(def_id)
    } else {
        None
    }
}

/// The state that is maintained for every local variable in the mir body
#[derive(Debug, Clone, Copy)]
enum LocalState {
    /// Local is live, no scope is borrowed at this point
    LiveBeforeBorrow,
    /// A mutable borrow to a scope has been taken and the local was killed as a result,
    /// and may never be used again
    Killed { at: Span },
}

struct TraverseCtxt<'a, 'tcx> {
    ocx: ObligationCtxt<'a, 'tcx>,
    param_env: ParamEnv<'tcx>,
    tcx: TyCtxt<'tcx>,
    fn_def_id: LocalDefId,
    lint: &'a mut MissingRoot,
    visited_bbs: BitSet<BasicBlock>,
}

impl<'tcx> TraverseCtxt<'_, 'tcx> {
    fn is_rootable(&self, ty: Ty<'tcx>) -> bool {
        let trait_ref = ty::TraitRef::new(self.tcx, self.lint.root_trait(), [ty::GenericArg::from(ty)]);
        let obligation = Obligation::new(self.tcx, ObligationCause::dummy(), self.param_env, trait_ref);
        self.ocx.register_obligation(obligation);
        self.ocx.select_all_or_error().is_empty()
    }
}

/// Additional state when processing a single basic block.
/// That state is then joined when merging two basic blocks.
#[derive(Debug, Clone)]
struct TraverseState {
    locals: IndexVec<Local, LocalState>,
}

impl TraverseState {
    fn kill(&mut self, at: Span) {
        self.locals.raw.fill(LocalState::Killed { at });
    }

    fn join(default: Self, states: impl IntoIterator<Item = TraverseState>) -> TraverseState {
        let mut states = states.into_iter();
        let Some(mut joined_state) = states.next() else {
            return default;
        };

        for state in states {
            for (joined_state, state) in iter::zip(&mut joined_state.locals, state.locals) {
                *joined_state = match (*joined_state, state) {
                    (LocalState::LiveBeforeBorrow, LocalState::LiveBeforeBorrow) => LocalState::LiveBeforeBorrow,
                    (LocalState::LiveBeforeBorrow, LocalState::Killed { at })
                    | (LocalState::Killed { at }, LocalState::LiveBeforeBorrow) => LocalState::Killed { at },
                    (LocalState::Killed { at }, LocalState::Killed { at: _ }) => LocalState::Killed { at },
                };
            }
        }

        joined_state
    }
}

struct PlaceVisitor<'a, 'b, 'tcx> {
    span: Span,
    cx: &'a mut TraverseCtxt<'b, 'tcx>,
    mir: &'a Body<'tcx>,
    state: &'a mut TraverseState,
}

impl<'a, 'tcx> mir::visit::Visitor<'tcx> for PlaceVisitor<'a, '_, 'tcx> {
    fn visit_place(&mut self, place: &mir::Place<'tcx>, context: mir::visit::PlaceContext, location: mir::Location) {
        let ty = self.mir.local_decls[place.local].ty;

        if self.cx.lint.is_vm_like(ty) {
            self.state.kill(self.span);
        } else if self.cx.is_rootable(ty)
            && let LocalState::Killed { at: killed_at } = self.state.locals[place.local]
        {
            if context.is_place_assignment() {
                // Assigning to an `Unrooted` revives itself
                self.state.locals[place.local] = LocalState::LiveBeforeBorrow;
            } else {
                let hir_id = self.cx.tcx.local_def_id_to_hir_id(self.cx.fn_def_id);
                self.cx.tcx.node_span_lint(MISSING_ROOT, hir_id, self.span, |diag| {
                    diag.primary_message("use of unrooted value after mutable scope borrow");
                    diag.span_note(killed_at, "scope mutably borrowed here");
                });
            }
        }

        self.super_place(place, context, location)
    }
}

fn traverse<'tcx>(
    cx: &mut TraverseCtxt<'_, 'tcx>,
    mir: &Body<'tcx>,
    mut state: TraverseState,
    bb: BasicBlock,
) -> TraverseState {
    if !cx.visited_bbs.insert(bb) {
        // TODO: is returning this for block cycles correct?
        return state;
    }

    for (statement_index, stmt) in mir.basic_blocks[bb].statements.iter().enumerate() {
        let vis = &mut PlaceVisitor {
            cx,
            mir,
            state: &mut state,
            span: stmt.source_info.span,
        };
        let location = mir::Location {
            block: bb,
            statement_index,
        };

        if let mir::StatementKind::Assign(box (_, value)) = &stmt.kind
            && let mir::Rvalue::Ref(_, mir::BorrowKind::Mut { .. }, place) = value
            && vis.cx.lint.is_vm_like(mir.local_decls[place.local].ty)
        {
            // Special case `_x = &mut (*_y)` where `*y` is vm-like to not kill.
            // That is itself part of special casing `.root(sc)`, which is split into multiple statements.
        } else {
            stmt.apply(location, vis);
        }
    }

    let terminator = mir.basic_blocks[bb].terminator();

    if let mir::TerminatorKind::Call {
        func: mir::Operand::Constant(box func),
        ..
    } = &terminator.kind
        && let ty::FnDef(def_id, ..) = *func.ty().kind()
        && cx.tcx.trait_of_item(def_id) == Some(cx.lint.root_trait())
    {
        // Don't look for mutating uses of vms in .root() calls because they are specifically ok
    } else {
        terminator.apply(
            mir::Location {
                block: bb,
                statement_index: mir.basic_blocks[bb].statements.len(),
            },
            &mut PlaceVisitor {
                cx,
                mir,
                state: &mut state,
                span: terminator.source_info.span,
            },
        );
    }

    match terminator.edges() {
        mir::TerminatorEdges::None => state,
        mir::TerminatorEdges::Single(bb) => traverse(cx, mir, state, bb),
        mir::TerminatorEdges::Double(bb1, bb2) => TraverseState::join(
            state.clone(),
            [
                traverse(cx, mir, state.clone(), bb1),
                traverse(cx, mir, state.clone(), bb2),
            ],
        ),
        mir::TerminatorEdges::AssignOnReturn {
            return_,
            cleanup,
            place: _,
        } => TraverseState::join(
            state.clone(),
            return_
                .iter()
                .copied()
                .chain(cleanup)
                .map(|bb| traverse(cx, mir, state.clone(), bb)),
        ),
        mir::TerminatorEdges::SwitchInt { targets, discr: _ } => TraverseState::join(
            state.clone(),
            targets
                .all_targets()
                .iter()
                .copied()
                .map(|bb| traverse(cx, mir, state.clone(), bb)),
        ),
    }
}

impl LateLintPass<'_> for MissingRoot {
    fn check_crate(&mut self, cx: &LateContext<'_>) {
        if let Some(root) = unique_res(cx, &["dash_vm", "value", "Root"], DefKind::Trait)
            && let Some(vm) = unique_res(cx, &["dash_vm", "Vm"], DefKind::Struct)
            && let Some(scope) = unique_res(cx, &["dash_vm", "localscope", "LocalScope"], DefKind::Struct)
        {
            self.root = Some(root);
            self.vm = Some(vm);
            self.scope = Some(scope);
        } else {
            bug!("failed to get required items")
        }
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'_>,
        _: FnKind<'_>,
        _: &hir::FnDecl<'_>,
        _: &hir::Body<'_>,
        _: Span,
        def_id: LocalDefId,
    ) {
        let mir = cx.tcx.optimized_mir(def_id);

        let locals = IndexVec::<Local, LocalState>::from_elem_n(LocalState::LiveBeforeBorrow, mir.local_decls.len());

        let infcx = cx.tcx.infer_ctxt().build();
        let ocx = ObligationCtxt::new(&infcx);
        traverse(
            &mut TraverseCtxt {
                ocx,
                param_env: cx.param_env,
                tcx: cx.tcx,
                fn_def_id: def_id,
                lint: self,
                visited_bbs: BitSet::new_empty(mir.basic_blocks.len()),
            },
            mir,
            TraverseState { locals },
            BasicBlock::ZERO,
        );
    }
}
