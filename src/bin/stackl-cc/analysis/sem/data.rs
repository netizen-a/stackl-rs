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
	pub(super) fn dtype_eq(&self, lhs: &DataType, rhs: &DataType) -> bool {
		match (&lhs.kind, &rhs.kind) {
			(_, _) => todo!(),
		}
	}
}
