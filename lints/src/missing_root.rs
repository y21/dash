use std::iter;

use clippy_utils::def_path_res;
use mir::visit::MirVisitable as _;
use rustc_ast::Mutability;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::intravisit::FnKind;
use rustc_hir::{self as hir};
use rustc_index::bit_set::BitSet;
use rustc_index::IndexVec;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::{Obligation, ObligationCause};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::mir::tcx::PlaceTy;
use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::{self, BasicBlock, Body, Local};
use rustc_middle::ty::{ParamEnv, Ty, TyCtxt};
use rustc_middle::{bug, ty};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::source_map::Spanned;
use rustc_span::Span;
use rustc_target::abi::VariantIdx;
use rustc_trait_selection::traits::ObligationCtxt;

use crate::utils::has_no_gc_attr;

declare_lint! {
    pub MISSING_ROOT,
    Deny,
    "missing .root() calls"
}

macro_rules! required_items {
    ($($struct_or_trait:tt $field:ident => $respath:expr),+) => {
        macro_rules! defkind {
            (trait) => { DefKind::Trait };
            (struct) => { DefKind::Struct }
        }

        #[derive(Default)]
        struct RequiredItems {
            $($field: Option<DefId>),+
        }
        impl RequiredItems {
            fn init(&mut self, cx: &LateContext<'_>) {
                $(
                    self.$field = Some(unique_res(cx, &$respath, defkind!($struct_or_trait)));
                )+
            }

            $(
                fn $field(&self) -> DefId {
                    self.$field.unwrap()
                }
            )+
        }
    };
}
required_items!(
    trait root => ["dash_vm", "value", "Root"],
    struct vm => ["dash_vm", "Vm"],
    struct allocator => ["dash_vm", "gc", "Allocator"],
    struct scope => ["dash_vm", "localscope", "LocalScope"],
    struct dispatchcx => ["dash_vm", "dispatch", "DispatchContext"]
);

#[derive(Default)]
pub struct MissingRoot {
    items: RequiredItems,
}

impl_lint_pass!(MissingRoot => [MISSING_ROOT]);

#[derive(Copy, Clone)]
enum RefBehavior {
    OnlyMut,
    PermitImm,
}

impl MissingRoot {
    fn is_scope_like<'tcx>(&self, tcx: TyCtxt<'tcx>, ty: Ty<'tcx>, refb: RefBehavior) -> bool {
        match *ty.kind() {
            ty::Adt(def, _) => [
                self.items.dispatchcx(),
                self.items.vm(),
                self.items.scope(),
                self.items.allocator(),
            ]
            .contains(&def.did()),
            ty::Ref(_, pointee, Mutability::Mut) => self.is_scope_like(tcx, pointee, refb),
            ty::Ref(_, pointee, Mutability::Not) if let RefBehavior::PermitImm = refb => {
                self.is_scope_like(tcx, pointee, refb)
            }
            ty::Closure(closure_def_id, _) => tcx
                .closure_captures(closure_def_id.as_local().unwrap())
                .iter()
                .any(|capture| self.is_scope_like(tcx, capture.place.ty(), refb)),
            _ => false,
        }
    }
}

fn unique_res(cx: &LateContext<'_>, path: &[&str], kind: DefKind) -> DefId {
    let mut unrooted_res = def_path_res(cx.tcx, path).into_iter();
    if let Some(Res::Def(res_kind, def_id)) = unrooted_res.next()
        && res_kind == kind
        && unrooted_res.next().is_none()
    {
        def_id
    } else {
        bug!("failed to resolve {path:?}")
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
        let trait_ref = ty::TraitRef::new(self.tcx, self.lint.items.root(), [ty::GenericArg::from(ty.peel_refs())]);
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
        let base_ty = self.mir.local_decls[place.local].ty;

        if self.cx.is_rootable(base_ty)
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

        let mut kill = false;
        let mut projections = place.iter_projections();
        let mut ty = base_ty;

        for (_, projection) in &mut projections {
            ty = PlaceTy::from_ty(ty).projection_ty(self.cx.tcx, projection).ty;

            if self.cx.lint.is_scope_like(self.cx.tcx, ty, RefBehavior::OnlyMut) {
                kill = true;
                break;
            }
        }

        if kill {
            for (_, projection) in projections {
                if let mir::ProjectionElem::Field(idx, _) = projection
                    && let Some(adt) = ty.ty_adt_def()
                {
                    assert!(adt.is_struct());
                    let field = &adt.variant(VariantIdx::from_u32(0)).fields[idx];
                    if has_no_gc_attr(self.cx.tcx, field.did) {
                        kill = false;
                        continue;
                    }
                }

                kill = true;
            }
        }

        if kill {
            self.state.kill(self.span);
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

        if let mir::StatementKind::Assign(box (assignee, _)) = &stmt.kind
            && vis.cx.lint.is_scope_like(
                vis.cx.tcx,
                assignee.ty(&mir.local_decls, vis.cx.tcx).ty,
                RefBehavior::PermitImm,
            )
        {
            // Specifically allow assignments of scope-like things.
            // This is itself part of the special casing for fn call terminators down below
        } else {
            stmt.apply(location, vis);
        }
    }

    let terminator = mir.basic_blocks[bb].terminator();

    let vis = &mut PlaceVisitor {
        cx,
        mir,
        state: &mut state,
        span: terminator.source_info.span,
    };
    let location = mir::Location {
        block: bb,
        statement_index: mir.basic_blocks[bb].statements.len(),
    };

    if let mir::TerminatorKind::Call {
        func,
        args,
        destination,
        target: _,
        unwind: _,
        call_source: _,
        fn_span: _,
    } = &terminator.kind
    {
        if let mir::Operand::Constant(ct) = func
            && let ty::FnDef(callee_def_id, _) = *ct.ty().kind()
            && (vis.cx.tcx.trait_of_item(callee_def_id) == Some(vis.cx.tcx.lang_items().deref_mut_trait().unwrap())
                || has_no_gc_attr(vis.cx.tcx, callee_def_id))
        {
            // Allow derefs (and anything that is annotated with #[trusted_no_gc]).
            // They could in theory do arbitrary work, but in practice deref should never do that.
        } else {
            // General function calls of the form `fun(scope, arg2, arg3)` are special cased:
            // Even though `scope` of type `&mut LocalScope` is evaluated first, which would kill all unrooteds
            // as per the usual rules, we do want to special case scope references passed *directly* to functions:
            // - Only *after* evaluating the function call should all unrooteds be killed
            //   - This allows passing unrooteds along with the scope into a function
            // - The assignee, i.e. `unrooted2 = fun(scope, unrooted1)`, should NOT be killed

            vis.visit_operand(func, location);

            let mut kill = false;
            for Spanned { node, .. } in args {
                if let mir::Operand::Move(place) = node
                    && let ty = place.ty(&mir.local_decls, vis.cx.tcx).ty
                    && let ty::Ref(_, pointee, Mutability::Mut) = *ty.kind()
                    && vis.cx.lint.is_scope_like(vis.cx.tcx, pointee, RefBehavior::OnlyMut)
                {
                    kill = true;
                } else {
                    vis.visit_operand(node, location);
                }
            }

            if kill {
                vis.state.kill(terminator.source_info.span);
            }

            // Evaluate destination *after* the potential kill, which makes the destination live again
            vis.visit_place(
                destination,
                mir::visit::PlaceContext::MutatingUse(mir::visit::MutatingUseContext::Call),
                location,
            );
        }
    } else if let mir::TerminatorKind::Drop { .. } = terminator.kind {
        // Allow drops
    } else {
        terminator.apply(location, vis);
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
        self.items.init(cx);
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
