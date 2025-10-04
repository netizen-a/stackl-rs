use std::process::exit;

use crate::analysis::syn;
use crate::data_types::*;
use crate::diagnostics as diag;
use diag::ToSpan;

impl super::SemanticParser<'_> {
	pub(super) fn unwrap_or_poison(
		&mut self,
		value: Option<DataType>,
		name: Option<String>,
		span: diag::Span,
	) -> DataType {
		match value {
			Some(ty) => ty.clone(),
			None => {
				let diag = diag::Diagnostic::error(diag::DiagKind::ImplicitInt(name), span);
				self.diagnostics.push(diag);
				DataType {
					kind: TypeKind::Poison,
					qual: TypeQual::default(),
				}
			}
		}
	}
	pub(super) fn dtype_eq(&mut self, lhs: &DataType, rhs: &DataType) -> bool {
		match (&lhs.kind, &rhs.kind) {
			(TypeKind::Scalar(l_scalar), TypeKind::Scalar(r_scalar)) => l_scalar == r_scalar,
			(TypeKind::Pointer(l_ptr), TypeKind::Pointer(r_ptr)) => {
				self.dtype_eq(&l_ptr.0, &r_ptr.0)
			}
			(TypeKind::Pointer(ptr), TypeKind::Array(array)) => {
				if !array.is_decayed {
					return false;
				}
				self.dtype_eq(&ptr.0, &array.component)
			}
			(TypeKind::Array(array), TypeKind::Pointer(ptr)) => {
				if !array.is_decayed {
					return false;
				}
				self.dtype_eq(&array.component, &ptr.0)
			}
			(TypeKind::Poison, _) | (_, TypeKind::Poison) => {
				let kind = diag::DiagKind::Internal("compared poisoned data types!".to_string());
				let error = diag::Diagnostic::fatal(kind, None);
				self.diagnostics.push_and_exit(error);
			}
			(_, _) => false,
		}
	}
}
