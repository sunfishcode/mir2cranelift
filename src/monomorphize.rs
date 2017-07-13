use rustc::ty::subst::{Subst, Substs};
use rustc::ty::{TyCtxt, TypeFoldable};
use rustc::infer::TransNormalize;

pub fn apply_substs<'a, 'tcx, T>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                                 param_substs: &Substs<'tcx>,
                                 value: &T)
                                 -> T
    where T: TypeFoldable<'tcx> + TransNormalize<'tcx>
{
    let substituted = value.subst(tcx, param_substs);
    tcx.normalize_associated_type(&substituted)
}
