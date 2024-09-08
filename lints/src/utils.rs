use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;

pub fn item_has_attr(tcx: TyCtxt<'_>, def_id: DefId, attr: &str) -> bool {
    return tcx
        .get_attrs_by_path(def_id, &[Symbol::intern("dash_lints"), Symbol::intern(attr)])
        .next()
        .is_some();
}

pub fn has_no_gc_attr(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    item_has_attr(tcx, def_id, "trusted_no_gc")
}
