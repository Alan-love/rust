use super::USELESS_TRANSMUTE;
use crate::utils::{span_lint, span_lint_and_then, sugg};
use rustc_errors::Applicability;
use rustc_hir::Expr;
use rustc_lint::LateContext;
use rustc_middle::ty::{self, Ty};

/// Checks for `useless_transmute` lint.
/// Returns `true` if it's triggered, otherwise returns `false`.
pub(super) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    e: &'tcx Expr<'_>,
    from_ty: Ty<'tcx>,
    to_ty: Ty<'tcx>,
    args: &'tcx [Expr<'_>],
) -> bool {
    match (&from_ty.kind(), &to_ty.kind()) {
        _ if from_ty == to_ty => {
            span_lint(
                cx,
                USELESS_TRANSMUTE,
                e.span,
                &format!("transmute from a type (`{}`) to itself", from_ty),
            );
            true
        },
        (ty::Ref(_, rty, rty_mutbl), ty::RawPtr(ptr_ty)) => {
            span_lint_and_then(
                cx,
                USELESS_TRANSMUTE,
                e.span,
                "transmute from a reference to a pointer",
                |diag| {
                    if let Some(arg) = sugg::Sugg::hir_opt(cx, &args[0]) {
                        let rty_and_mut = ty::TypeAndMut {
                            ty: rty,
                            mutbl: *rty_mutbl,
                        };

                        let sugg = if *ptr_ty == rty_and_mut {
                            arg.as_ty(to_ty)
                        } else {
                            arg.as_ty(cx.tcx.mk_ptr(rty_and_mut)).as_ty(to_ty)
                        };

                        diag.span_suggestion(e.span, "try", sugg.to_string(), Applicability::Unspecified);
                    }
                },
            );
            true
        },
        (ty::Int(_) | ty::Uint(_), ty::RawPtr(_)) => {
            span_lint_and_then(
                cx,
                USELESS_TRANSMUTE,
                e.span,
                "transmute from an integer to a pointer",
                |diag| {
                    if let Some(arg) = sugg::Sugg::hir_opt(cx, &args[0]) {
                        diag.span_suggestion(
                            e.span,
                            "try",
                            arg.as_ty(&to_ty.to_string()).to_string(),
                            Applicability::Unspecified,
                        );
                    }
                },
            );
            true
        },
        _ => false,
    }
}
